use std::collections::HashMap;
use std::io::BufRead;
use std::net::TcpListener;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

use crate::course::types::{EnvCommand, EnvService, EnvironmentSpec, StateAssertion};
use crate::error::{LearnLocalError, Result};
use crate::exec::placeholder;
use crate::exec::sandbox::{Sandbox, StepOutput};

/// Result of setting up an environment in a sandbox.
#[derive(Debug, Default)]
pub struct SetupResult {
    pub env_vars: HashMap<String, String>,
    pub cwd_override: Option<PathBuf>,
}

/// Result of checking a single state assertion.
#[derive(Debug, Clone)]
pub struct AssertionResult {
    pub description: String,
    pub passed: bool,
    pub detail: String,
}

/// Validate that a path is safe for use inside a sandbox.
/// Rejects absolute paths and paths with `..` components.
fn validate_path_safety(path: &str) -> Result<()> {
    let p = Path::new(path);
    if p.is_absolute() {
        return Err(LearnLocalError::Execution(format!(
            "Absolute path not allowed in environment: {}",
            path
        )));
    }
    for component in p.components() {
        if matches!(component, Component::ParentDir) {
            return Err(LearnLocalError::Execution(format!(
                "Path with '..' not allowed in environment: {}",
                path
            )));
        }
    }
    Ok(())
}

/// Set up the sandbox environment according to the spec.
/// Creates dirs, files (with optional permissions), and symlinks.
/// Returns env vars and optional cwd override.
pub fn setup_environment(
    sandbox_dir: &Path,
    env_spec: &EnvironmentSpec,
    main_file: &str,
    all_files: &[String],
) -> Result<SetupResult> {
    // 1. Create directories first
    for dir in &env_spec.dirs {
        validate_path_safety(dir)?;
        let full_path = sandbox_dir.join(dir);
        std::fs::create_dir_all(&full_path).map_err(|e| {
            LearnLocalError::Execution(format!("Failed to create dir '{}': {}", dir, e))
        })?;
    }

    // 2. Create files (creates parent dirs as needed)
    for file in &env_spec.files {
        validate_path_safety(&file.path)?;
        let full_path = sandbox_dir.join(&file.path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                LearnLocalError::Execution(format!(
                    "Failed to create parent dir for '{}': {}",
                    file.path, e
                ))
            })?;
        }
        // Substitute {dir} placeholder in file content
        let content = file
            .content
            .replace("{dir}", &sandbox_dir.to_string_lossy());
        std::fs::write(&full_path, &content).map_err(|e| {
            LearnLocalError::Execution(format!("Failed to write file '{}': {}", file.path, e))
        })?;

        // Set permissions if specified (Unix only)
        #[cfg(unix)]
        if let Some(ref mode_str) = file.permissions {
            let mode = u32::from_str_radix(mode_str, 8).map_err(|_| {
                LearnLocalError::Execution(format!(
                    "Invalid octal permission '{}' for file '{}'",
                    mode_str, file.path
                ))
            })?;
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(mode);
            std::fs::set_permissions(&full_path, perms).map_err(|e| {
                LearnLocalError::Execution(format!(
                    "Failed to set permissions on '{}': {}",
                    file.path, e
                ))
            })?;
        }
    }

    // 3. Create symlinks (targets should exist by now, or intentionally broken)
    for symlink in &env_spec.symlinks {
        validate_path_safety(&symlink.link)?;
        validate_path_safety(&symlink.target)?;

        let link_path = sandbox_dir.join(&symlink.link);
        let target_path = PathBuf::from(&symlink.target);

        // Create parent dir for the symlink if needed
        if let Some(parent) = link_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                LearnLocalError::Execution(format!(
                    "Failed to create parent dir for symlink '{}': {}",
                    symlink.link, e
                ))
            })?;
        }

        #[cfg(unix)]
        std::os::unix::fs::symlink(&target_path, &link_path).map_err(|e| {
            LearnLocalError::Execution(format!(
                "Failed to create symlink '{}' -> '{}': {}",
                symlink.link, symlink.target, e
            ))
        })?;

        // Verify non-broken symlinks stay within sandbox
        #[cfg(unix)]
        {
            let full_target = sandbox_dir.join(&symlink.target);
            if full_target.exists() {
                if let Ok(canonical) = full_target.canonicalize() {
                    let sandbox_canonical = sandbox_dir
                        .canonicalize()
                        .unwrap_or_else(|_| sandbox_dir.to_path_buf());
                    if !canonical.starts_with(&sandbox_canonical) {
                        // Clean up the symlink we just created
                        let _ = std::fs::remove_file(&link_path);
                        return Err(LearnLocalError::Execution(format!(
                            "Symlink target '{}' resolves outside sandbox",
                            symlink.target
                        )));
                    }
                }
            }
            // Broken symlinks are allowed (for "fix the broken symlink" exercises)
        }
    }

    // 4. Build env vars with {dir} substitution
    let mut env_vars = HashMap::new();
    for (key, value) in &env_spec.env {
        let substituted = placeholder::substitute(value, sandbox_dir, main_file, all_files);
        env_vars.insert(key.clone(), substituted);
    }

    // 4b. Allocate dynamic ports if requested
    if env_spec.ports > 0 {
        let ports = allocate_ports(env_spec.ports)?;
        for (i, port) in ports.iter().enumerate() {
            env_vars.insert(format!("LEARNLOCAL_PORT_{}", i), port.to_string());
        }
    }

    // 5. Compute cwd override
    let cwd_override = if let Some(ref cwd) = env_spec.cwd {
        validate_path_safety(cwd)?;
        let cwd_path = sandbox_dir.join(cwd);
        // Ensure the cwd directory exists
        std::fs::create_dir_all(&cwd_path).map_err(|e| {
            LearnLocalError::Execution(format!("Failed to create cwd dir '{}': {}", cwd, e))
        })?;
        Some(cwd_path)
    } else {
        None
    };

    Ok(SetupResult {
        env_vars,
        cwd_override,
    })
}

/// Allocate N ephemeral ports by binding to 127.0.0.1:0 and recording the assigned port.
/// The listener is dropped immediately, freeing the port for use by services/student code.
/// TOCTOU race is acceptable for a tutorial framework.
fn allocate_ports(count: usize) -> Result<Vec<u16>> {
    let mut ports = Vec::with_capacity(count);
    for i in 0..count {
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| {
            LearnLocalError::Execution(format!("Failed to allocate port {}: {}", i, e))
        })?;
        let port = listener
            .local_addr()
            .map_err(|e| LearnLocalError::Execution(format!("Failed to get port address: {}", e)))?
            .port();
        ports.push(port);
        // Drop listener — port is now free for use
    }
    Ok(ports)
}

/// Run a single environment command (setup or teardown step).
/// Substitutes `{dir}` placeholder in command and args.
pub fn run_env_command(
    sandbox: &Sandbox,
    step: &EnvCommand,
    env_vars: Option<&HashMap<String, String>>,
    cwd_override: Option<&Path>,
    default_timeout: u64,
) -> Result<StepOutput> {
    let dir = sandbox.dir();
    // Only {dir} is meaningful for setup/teardown — student files don't exist yet for setup
    let command = step.command.replace("{dir}", &dir.to_string_lossy());
    let args: Vec<String> = step
        .args
        .iter()
        .map(|a| a.replace("{dir}", &dir.to_string_lossy()))
        .collect();

    let timeout = step.timeout_seconds.unwrap_or(default_timeout);

    sandbox.run_command_with_timeout(
        &command,
        &args,
        step.stdin.as_deref(),
        env_vars,
        cwd_override,
        timeout,
    )
}

/// Run a single environment command with full placeholder substitution.
/// Used for teardown commands where student files exist.
/// Substitutes `{dir}`, `{main}`, `{output}`, and `{files}` placeholders.
pub fn run_env_command_full(
    sandbox: &Sandbox,
    step: &EnvCommand,
    env_vars: Option<&HashMap<String, String>>,
    cwd_override: Option<&Path>,
    default_timeout: u64,
    main_file: &str,
    all_files: &[String],
) -> Result<StepOutput> {
    let command = placeholder::substitute(&step.command, sandbox.dir(), main_file, all_files);
    let args: Vec<String> = step
        .args
        .iter()
        .map(|a| placeholder::substitute(a, sandbox.dir(), main_file, all_files))
        .collect();

    let timeout = step.timeout_seconds.unwrap_or(default_timeout);

    sandbox.run_command_with_timeout(
        &command,
        &args,
        step.stdin.as_deref(),
        env_vars,
        cwd_override,
        timeout,
    )
}

/// Spawn a reader thread that watches a stream for a regex match, sending the result on `tx`.
/// Continues draining after match to prevent pipe backup.
/// If `capture_path` is set, writes all output to that file when the stream closes.
/// Returns a JoinHandle so callers can wait for the capture file to be written.
fn spawn_stream_reader<R: std::io::Read + Send + 'static>(
    stream: R,
    re: regex::Regex,
    tx: std::sync::mpsc::Sender<std::result::Result<(), String>>,
    svc_name: String,
    stream_label: &'static str,
    capture_path: Option<std::path::PathBuf>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let reader = std::io::BufReader::new(stream);
        let mut matched = false;
        let mut captured = if capture_path.is_some() {
            Some(String::new())
        } else {
            None
        };
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if !matched && re.is_match(&line) {
                        matched = true;
                        let _ = tx.send(Ok(()));
                    }
                    // Capture line if capturing
                    if let Some(ref mut buf) = captured {
                        buf.push_str(&line);
                        buf.push('\n');
                    }
                    // Continue draining to prevent pipe backup
                }
                Err(e) => {
                    if !matched {
                        let _ = tx.send(Err(format!(
                            "Service '{}': {} read error: {}",
                            svc_name, stream_label, e
                        )));
                    }
                    break;
                }
            }
        }
        // If we never matched, send error (receiver may already have a match from the other stream)
        if !matched {
            let _ = tx.send(Err(format!(
                "Service '{}': {} closed without matching ready pattern",
                svc_name, stream_label
            )));
        }
        // Write captured output to file
        if let (Some(path), Some(content)) = (capture_path, captured) {
            let _ = std::fs::write(path, content);
        }
    })
}

/// Wait for a background service to become ready.
///
/// Two modes:
/// - **Pattern mode** (`ready_pattern` set): reads stdout and/or stderr lines until one matches
///   the regex, or times out after `ready_timeout_seconds`. Controlled by `ready_stream`
///   ("stdout", "stderr", or "both"; default "both").
/// - **Delay mode** (no pattern): sleeps `ready_delay_ms`, then checks the process hasn't crashed.
/// Wait for a background service to become ready, returning join handles for reader threads.
///
/// The returned handles must be joined after killing the service to ensure capture files are written.
/// `sandbox_dir` is needed to resolve capture paths for streams consumed by readiness readers.
pub fn wait_for_service_ready(
    child: &mut std::process::Child,
    service: &EnvService,
    sandbox_dir: &Path,
) -> std::result::Result<Vec<std::thread::JoinHandle<()>>, LearnLocalError> {
    let mut reader_handles = Vec::new();

    if let Some(ref pattern) = service.ready_pattern {
        let re = regex::Regex::new(pattern).map_err(|e| {
            LearnLocalError::Execution(format!(
                "Service '{}': invalid ready_pattern '{}': {}",
                service.name, pattern, e
            ))
        })?;

        let watch_stream = service.ready_stream.as_deref().unwrap_or("both");

        let (tx, rx) = std::sync::mpsc::channel();

        // Watch stdout unless ready_stream is "stderr"
        if watch_stream != "stderr" {
            if let Some(stdout) = child.stdout.take() {
                let capture = service.capture_stdout.as_ref().map(|p| sandbox_dir.join(p));
                reader_handles.push(spawn_stream_reader(
                    stdout,
                    re.clone(),
                    tx.clone(),
                    service.name.clone(),
                    "stdout",
                    capture,
                ));
            }
        }

        // Watch stderr unless ready_stream is "stdout"
        if watch_stream != "stdout" {
            if let Some(stderr) = child.stderr.take() {
                let capture = service.capture_stderr.as_ref().map(|p| sandbox_dir.join(p));
                reader_handles.push(spawn_stream_reader(
                    stderr,
                    re.clone(),
                    tx.clone(),
                    service.name.clone(),
                    "stderr",
                    capture,
                ));
            }
        }

        // Drop original sender so channel closes when all threads finish
        drop(tx);

        let timeout = Duration::from_secs(service.ready_timeout_seconds);
        match rx.recv_timeout(timeout) {
            Ok(Ok(())) => Ok(reader_handles),
            Ok(Err(msg)) => {
                // Stream closed without match — but the other stream may still match.
                // Wait a bit more for the other stream's result.
                match rx.recv_timeout(Duration::from_millis(500)) {
                    Ok(Ok(())) => Ok(reader_handles),
                    _ => Err(LearnLocalError::Execution(msg)),
                }
            }
            Err(_) => Err(LearnLocalError::Execution(format!(
                "Service '{}': timed out after {}s waiting for ready pattern '{}'",
                service.name, service.ready_timeout_seconds, pattern
            ))),
        }
    } else {
        // Delay mode: sleep then check alive
        std::thread::sleep(Duration::from_millis(service.ready_delay_ms));
        match child.try_wait() {
            Ok(Some(status)) => Err(LearnLocalError::Execution(format!(
                "Service '{}': exited immediately with {}",
                service.name, status
            ))),
            Ok(None) => Ok(reader_handles), // Still running, good
            Err(e) => Err(LearnLocalError::Execution(format!(
                "Service '{}': failed to check status: {}",
                service.name, e
            ))),
        }
    }
}

/// Validate state assertions against the sandbox filesystem after exercise execution.
pub fn validate_state(sandbox_dir: &Path, assertions: &[StateAssertion]) -> Vec<AssertionResult> {
    assertions
        .iter()
        .map(|a| check_assertion(sandbox_dir, a))
        .collect()
}

/// Extract the primary path from a state assertion for safety checking.
fn assertion_path(assertion: &StateAssertion) -> Option<&str> {
    match assertion {
        StateAssertion::FileExists(p)
        | StateAssertion::DirExists(p)
        | StateAssertion::FileNotExists(p)
        | StateAssertion::DirNotExists(p)
        | StateAssertion::DirEmpty(p) => Some(p),
        StateAssertion::FileContains(c) | StateAssertion::FileEquals(c) => Some(&c.path),
        StateAssertion::FileMatches(c) => Some(&c.path),
        StateAssertion::Permissions(c) => Some(&c.path),
        StateAssertion::Symlink(c) => Some(&c.path),
        StateAssertion::FileCount(c) => Some(&c.path),
    }
}

fn check_assertion(sandbox_dir: &Path, assertion: &StateAssertion) -> AssertionResult {
    // Runtime path safety check — reject absolute and traversal paths
    if let Some(path) = assertion_path(assertion) {
        if let Err(e) = validate_path_safety(path) {
            return AssertionResult {
                description: format!("path safety: {}", path),
                passed: false,
                detail: format!("unsafe path: {}", e),
            };
        }
    }

    match assertion {
        StateAssertion::FileExists(path) => {
            let full = sandbox_dir.join(path);
            let exists = full.is_file();
            AssertionResult {
                description: format!("file_exists: {}", path),
                passed: exists,
                detail: if exists {
                    "file exists".to_string()
                } else {
                    "file not found".to_string()
                },
            }
        }
        StateAssertion::DirExists(path) => {
            let full = sandbox_dir.join(path);
            let exists = full.is_dir();
            AssertionResult {
                description: format!("dir_exists: {}", path),
                passed: exists,
                detail: if exists {
                    "directory exists".to_string()
                } else {
                    "directory not found".to_string()
                },
            }
        }
        StateAssertion::FileNotExists(path) => {
            let full = sandbox_dir.join(path);
            let gone = !full.exists();
            AssertionResult {
                description: format!("file_not_exists: {}", path),
                passed: gone,
                detail: if gone {
                    "file does not exist".to_string()
                } else {
                    "file still exists".to_string()
                },
            }
        }
        StateAssertion::DirNotExists(path) => {
            let full = sandbox_dir.join(path);
            let gone = !full.exists();
            AssertionResult {
                description: format!("dir_not_exists: {}", path),
                passed: gone,
                detail: if gone {
                    "directory does not exist".to_string()
                } else {
                    "directory still exists".to_string()
                },
            }
        }
        StateAssertion::FileContains(check) => {
            let full = sandbox_dir.join(&check.path);
            match std::fs::read_to_string(&full) {
                Ok(content) => {
                    let contains = content.contains(&check.content);
                    AssertionResult {
                        description: format!("file_contains: {}", check.path),
                        passed: contains,
                        detail: if contains {
                            "content found".to_string()
                        } else {
                            format!("'{}' not found in file", check.content)
                        },
                    }
                }
                Err(e) => AssertionResult {
                    description: format!("file_contains: {}", check.path),
                    passed: false,
                    detail: format!("cannot read file: {}", e),
                },
            }
        }
        StateAssertion::FileMatches(check) => {
            let full = sandbox_dir.join(&check.path);
            match std::fs::read_to_string(&full) {
                Ok(content) => match regex::Regex::new(&check.pattern) {
                    Ok(re) => {
                        let matches = re.is_match(&content);
                        AssertionResult {
                            description: format!("file_matches: {}", check.path),
                            passed: matches,
                            detail: if matches {
                                "pattern matched".to_string()
                            } else {
                                format!("pattern /{} / not matched", check.pattern)
                            },
                        }
                    }
                    Err(e) => AssertionResult {
                        description: format!("file_matches: {}", check.path),
                        passed: false,
                        detail: format!("invalid regex: {}", e),
                    },
                },
                Err(e) => AssertionResult {
                    description: format!("file_matches: {}", check.path),
                    passed: false,
                    detail: format!("cannot read file: {}", e),
                },
            }
        }
        StateAssertion::FileEquals(check) => {
            let full = sandbox_dir.join(&check.path);
            match std::fs::read_to_string(&full) {
                Ok(content) => {
                    let equals = content.trim() == check.content.trim();
                    AssertionResult {
                        description: format!("file_equals: {}", check.path),
                        passed: equals,
                        detail: if equals {
                            "content matches".to_string()
                        } else {
                            format!(
                                "expected '{}', got '{}'",
                                check.content.trim(),
                                content.trim()
                            )
                        },
                    }
                }
                Err(e) => AssertionResult {
                    description: format!("file_equals: {}", check.path),
                    passed: false,
                    detail: format!("cannot read file: {}", e),
                },
            }
        }
        StateAssertion::Permissions(check) => {
            #[cfg(unix)]
            {
                let full = sandbox_dir.join(&check.path);
                match std::fs::metadata(&full) {
                    Ok(meta) => {
                        use std::os::unix::fs::PermissionsExt;
                        let actual_mode = meta.permissions().mode() & 0o777;
                        let expected_mode = u32::from_str_radix(&check.mode, 8).unwrap_or(0);
                        let passed = actual_mode == expected_mode;
                        AssertionResult {
                            description: format!("permissions: {} = {}", check.path, check.mode),
                            passed,
                            detail: if passed {
                                format!("mode {:o}", actual_mode)
                            } else {
                                format!("expected {:o}, actual {:o}", expected_mode, actual_mode)
                            },
                        }
                    }
                    Err(e) => AssertionResult {
                        description: format!("permissions: {}", check.path),
                        passed: false,
                        detail: format!("cannot stat file: {}", e),
                    },
                }
            }
            #[cfg(not(unix))]
            {
                AssertionResult {
                    description: format!("permissions: {}", check.path),
                    passed: true,
                    detail: "permissions check skipped (not unix)".to_string(),
                }
            }
        }
        StateAssertion::Symlink(check) => {
            let full = sandbox_dir.join(&check.path);
            match std::fs::read_link(&full) {
                Ok(target) => {
                    let target_str = target.to_string_lossy();
                    let passed = target_str == check.target;
                    AssertionResult {
                        description: format!("symlink: {} -> {}", check.path, check.target),
                        passed,
                        detail: if passed {
                            "symlink target correct".to_string()
                        } else {
                            format!("expected '{}', actual '{}'", check.target, target_str)
                        },
                    }
                }
                Err(e) => AssertionResult {
                    description: format!("symlink: {}", check.path),
                    passed: false,
                    detail: format!("not a symlink or cannot read: {}", e),
                },
            }
        }
        StateAssertion::FileCount(check) => {
            let full = sandbox_dir.join(&check.path);
            match std::fs::read_dir(&full) {
                Ok(entries) => {
                    let count = entries.filter_map(|e| e.ok()).count();
                    let passed = count == check.count;
                    AssertionResult {
                        description: format!("file_count: {} = {}", check.path, check.count),
                        passed,
                        detail: if passed {
                            format!("{} entries", count)
                        } else {
                            format!("expected {} entries, found {}", check.count, count)
                        },
                    }
                }
                Err(e) => AssertionResult {
                    description: format!("file_count: {}", check.path),
                    passed: false,
                    detail: format!("cannot read directory: {}", e),
                },
            }
        }
        StateAssertion::DirEmpty(path) => {
            let full = sandbox_dir.join(path);
            match std::fs::read_dir(&full) {
                Ok(mut entries) => {
                    let empty = entries.next().is_none();
                    AssertionResult {
                        description: format!("dir_empty: {}", path),
                        passed: empty,
                        detail: if empty {
                            "directory is empty".to_string()
                        } else {
                            "directory is not empty".to_string()
                        },
                    }
                }
                Err(e) => AssertionResult {
                    description: format!("dir_empty: {}", path),
                    passed: false,
                    detail: format!("cannot read directory: {}", e),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::course::types::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_path_safety_rejects_absolute() {
        assert!(validate_path_safety("/etc/passwd").is_err());
    }

    #[test]
    fn test_validate_path_safety_rejects_dotdot() {
        assert!(validate_path_safety("../escape").is_err());
        assert!(validate_path_safety("dir/../escape").is_err());
    }

    #[test]
    fn test_validate_path_safety_accepts_relative() {
        assert!(validate_path_safety("data/input.txt").is_ok());
        assert!(validate_path_safety("output").is_ok());
        assert!(validate_path_safety("a/b/c.txt").is_ok());
    }

    #[test]
    fn test_setup_creates_dirs() {
        let tmp = TempDir::new().unwrap();
        let spec = EnvironmentSpec {
            dirs: vec!["output".to_string(), "output/logs".to_string()],
            ..Default::default()
        };
        setup_environment(tmp.path(), &spec, "main.sh", &["main.sh".to_string()]).unwrap();
        assert!(tmp.path().join("output").is_dir());
        assert!(tmp.path().join("output/logs").is_dir());
    }

    #[test]
    fn test_setup_creates_files() {
        let tmp = TempDir::new().unwrap();
        let spec = EnvironmentSpec {
            files: vec![EnvFile {
                path: "data/input.txt".to_string(),
                content: "hello world".to_string(),
                permissions: None,
            }],
            ..Default::default()
        };
        setup_environment(tmp.path(), &spec, "main.sh", &["main.sh".to_string()]).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("data/input.txt")).unwrap();
        assert_eq!(content, "hello world");
    }

    #[cfg(unix)]
    #[test]
    fn test_setup_file_permissions() {
        let tmp = TempDir::new().unwrap();
        let spec = EnvironmentSpec {
            files: vec![EnvFile {
                path: "script.sh".to_string(),
                content: "#!/bin/bash".to_string(),
                permissions: Some("755".to_string()),
            }],
            ..Default::default()
        };
        setup_environment(tmp.path(), &spec, "main.sh", &["main.sh".to_string()]).unwrap();
        use std::os::unix::fs::PermissionsExt;
        let meta = std::fs::metadata(tmp.path().join("script.sh")).unwrap();
        assert_eq!(meta.permissions().mode() & 0o777, 0o755);
    }

    #[cfg(unix)]
    #[test]
    fn test_setup_creates_symlinks() {
        let tmp = TempDir::new().unwrap();
        let spec = EnvironmentSpec {
            files: vec![EnvFile {
                path: "data/v1.txt".to_string(),
                content: "version 1".to_string(),
                permissions: None,
            }],
            symlinks: vec![EnvSymlink {
                link: "latest".to_string(),
                target: "data/v1.txt".to_string(),
            }],
            ..Default::default()
        };
        setup_environment(tmp.path(), &spec, "main.sh", &["main.sh".to_string()]).unwrap();
        let link_path = tmp.path().join("latest");
        assert!(link_path
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    fn test_allocate_ports() {
        let ports = allocate_ports(3).unwrap();
        assert_eq!(ports.len(), 3);
        // All ports should be non-zero and unique
        for &p in &ports {
            assert!(p > 0);
        }
        let unique: std::collections::HashSet<_> = ports.iter().collect();
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn test_setup_with_ports() {
        let tmp = TempDir::new().unwrap();
        let spec = EnvironmentSpec {
            ports: 2,
            ..Default::default()
        };
        let result =
            setup_environment(tmp.path(), &spec, "main.sh", &["main.sh".to_string()]).unwrap();
        assert!(result.env_vars.contains_key("LEARNLOCAL_PORT_0"));
        assert!(result.env_vars.contains_key("LEARNLOCAL_PORT_1"));
        assert!(!result.env_vars.contains_key("LEARNLOCAL_PORT_2"));
        // Values should parse as valid ports
        let port0: u16 = result.env_vars["LEARNLOCAL_PORT_0"].parse().unwrap();
        assert!(port0 > 0);
    }

    #[test]
    fn test_setup_zero_ports() {
        let tmp = TempDir::new().unwrap();
        let spec = EnvironmentSpec {
            ports: 0,
            ..Default::default()
        };
        let result =
            setup_environment(tmp.path(), &spec, "main.sh", &["main.sh".to_string()]).unwrap();
        assert!(!result.env_vars.contains_key("LEARNLOCAL_PORT_0"));
    }

    #[test]
    fn test_setup_file_content_dir_substitution() {
        let tmp = TempDir::new().unwrap();
        let spec = EnvironmentSpec {
            files: vec![EnvFile {
                path: "config.txt".to_string(),
                content: "workdir={dir}".to_string(),
                permissions: None,
            }],
            ..Default::default()
        };
        setup_environment(tmp.path(), &spec, "main.sh", &["main.sh".to_string()]).unwrap();
        let content = std::fs::read_to_string(tmp.path().join("config.txt")).unwrap();
        assert!(content.contains(&tmp.path().to_string_lossy().to_string()));
        assert!(!content.contains("{dir}"));
    }

    #[test]
    fn test_setup_env_vars_substitution() {
        let tmp = TempDir::new().unwrap();
        let mut env = HashMap::new();
        env.insert("WORKDIR".to_string(), "{dir}".to_string());
        let spec = EnvironmentSpec {
            env,
            ..Default::default()
        };
        let result =
            setup_environment(tmp.path(), &spec, "main.sh", &["main.sh".to_string()]).unwrap();
        assert_eq!(
            result.env_vars.get("WORKDIR").unwrap(),
            &tmp.path().to_string_lossy().to_string()
        );
    }

    #[test]
    fn test_setup_cwd_override() {
        let tmp = TempDir::new().unwrap();
        let spec = EnvironmentSpec {
            dirs: vec!["workdir".to_string()],
            cwd: Some("workdir".to_string()),
            ..Default::default()
        };
        let result =
            setup_environment(tmp.path(), &spec, "main.sh", &["main.sh".to_string()]).unwrap();
        assert_eq!(result.cwd_override, Some(tmp.path().join("workdir")));
    }

    #[test]
    fn test_setup_rejects_absolute_path() {
        let tmp = TempDir::new().unwrap();
        let spec = EnvironmentSpec {
            dirs: vec!["/etc".to_string()],
            ..Default::default()
        };
        assert!(setup_environment(tmp.path(), &spec, "main.sh", &["main.sh".to_string()]).is_err());
    }

    #[test]
    fn test_setup_rejects_dotdot_path() {
        let tmp = TempDir::new().unwrap();
        let spec = EnvironmentSpec {
            files: vec![EnvFile {
                path: "../escape.txt".to_string(),
                content: "bad".to_string(),
                permissions: None,
            }],
            ..Default::default()
        };
        assert!(setup_environment(tmp.path(), &spec, "main.sh", &["main.sh".to_string()]).is_err());
    }

    #[test]
    fn test_validate_file_exists() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("test.txt"), "hello").unwrap();

        let results = validate_state(
            tmp.path(),
            &[StateAssertion::FileExists("test.txt".to_string())],
        );
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);

        let results = validate_state(
            tmp.path(),
            &[StateAssertion::FileExists("missing.txt".to_string())],
        );
        assert!(!results[0].passed);
    }

    #[test]
    fn test_validate_dir_exists() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join("subdir")).unwrap();

        let results = validate_state(
            tmp.path(),
            &[StateAssertion::DirExists("subdir".to_string())],
        );
        assert!(results[0].passed);
    }

    #[test]
    fn test_validate_file_not_exists() {
        let tmp = TempDir::new().unwrap();
        let results = validate_state(
            tmp.path(),
            &[StateAssertion::FileNotExists("gone.txt".to_string())],
        );
        assert!(results[0].passed);

        std::fs::write(tmp.path().join("still-here.txt"), "x").unwrap();
        let results = validate_state(
            tmp.path(),
            &[StateAssertion::FileNotExists("still-here.txt".to_string())],
        );
        assert!(!results[0].passed);
    }

    #[test]
    fn test_validate_file_contains() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("report.txt"), "Total: 55 items").unwrap();

        let results = validate_state(
            tmp.path(),
            &[StateAssertion::FileContains(FileContentCheck {
                path: "report.txt".to_string(),
                content: "Total: 55".to_string(),
            })],
        );
        assert!(results[0].passed);

        let results = validate_state(
            tmp.path(),
            &[StateAssertion::FileContains(FileContentCheck {
                path: "report.txt".to_string(),
                content: "Total: 99".to_string(),
            })],
        );
        assert!(!results[0].passed);
    }

    #[test]
    fn test_validate_file_equals() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("result.txt"), "42\n").unwrap();

        let results = validate_state(
            tmp.path(),
            &[StateAssertion::FileEquals(FileContentCheck {
                path: "result.txt".to_string(),
                content: "42".to_string(),
            })],
        );
        assert!(results[0].passed);
    }

    #[test]
    fn test_validate_file_matches() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("log.txt"), "processed 42 items").unwrap();

        let results = validate_state(
            tmp.path(),
            &[StateAssertion::FileMatches(FilePatternCheck {
                path: "log.txt".to_string(),
                pattern: r"\d+ items".to_string(),
            })],
        );
        assert!(results[0].passed);
    }

    #[cfg(unix)]
    #[test]
    fn test_validate_permissions() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("script.sh");
        std::fs::write(&path, "#!/bin/bash").unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();

        let results = validate_state(
            tmp.path(),
            &[StateAssertion::Permissions(PermissionsCheck {
                path: "script.sh".to_string(),
                mode: "755".to_string(),
            })],
        );
        assert!(results[0].passed);

        let results = validate_state(
            tmp.path(),
            &[StateAssertion::Permissions(PermissionsCheck {
                path: "script.sh".to_string(),
                mode: "644".to_string(),
            })],
        );
        assert!(!results[0].passed);
    }

    #[cfg(unix)]
    #[test]
    fn test_validate_symlink() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("target.txt"), "data").unwrap();
        std::os::unix::fs::symlink("target.txt", tmp.path().join("link")).unwrap();

        let results = validate_state(
            tmp.path(),
            &[StateAssertion::Symlink(SymlinkCheck {
                path: "link".to_string(),
                target: "target.txt".to_string(),
            })],
        );
        assert!(results[0].passed);
    }

    #[test]
    fn test_validate_file_count() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("output");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("a.txt"), "a").unwrap();
        std::fs::write(dir.join("b.txt"), "b").unwrap();

        let results = validate_state(
            tmp.path(),
            &[StateAssertion::FileCount(FileCountCheck {
                path: "output".to_string(),
                count: 2,
            })],
        );
        assert!(results[0].passed);

        let results = validate_state(
            tmp.path(),
            &[StateAssertion::FileCount(FileCountCheck {
                path: "output".to_string(),
                count: 3,
            })],
        );
        assert!(!results[0].passed);
    }

    #[test]
    fn test_validate_dir_empty() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join("empty")).unwrap();

        let results = validate_state(tmp.path(), &[StateAssertion::DirEmpty("empty".to_string())]);
        assert!(results[0].passed);

        std::fs::write(tmp.path().join("empty/file.txt"), "x").unwrap();
        let results = validate_state(tmp.path(), &[StateAssertion::DirEmpty("empty".to_string())]);
        assert!(!results[0].passed);
    }

    #[test]
    fn test_validate_multiple_assertions() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir(tmp.path().join("output")).unwrap();
        std::fs::write(tmp.path().join("output/report.txt"), "Total: 55").unwrap();

        let results = validate_state(
            tmp.path(),
            &[
                StateAssertion::DirExists("output".to_string()),
                StateAssertion::FileExists("output/report.txt".to_string()),
                StateAssertion::FileContains(FileContentCheck {
                    path: "output/report.txt".to_string(),
                    content: "Total: 55".to_string(),
                }),
                StateAssertion::FileNotExists("missing.txt".to_string()),
            ],
        );
        assert_eq!(results.len(), 4);
        assert!(results.iter().all(|r| r.passed));
    }

    #[test]
    fn test_run_env_command_basic() {
        use crate::course::types::ExecutionLimits;
        use crate::exec::sandbox::{Sandbox, SandboxLevel};

        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        let step = EnvCommand {
            name: "create-file".to_string(),
            command: "sh".to_string(),
            args: vec!["-c".to_string(), "echo hello > {dir}/test.txt".to_string()],
            stdin: None,
            timeout_seconds: Some(5),
            capture_to: None,
        };
        let result = run_env_command(&sandbox, &step, None, None, 10).unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(sandbox.dir().join("test.txt").exists());
    }

    #[test]
    fn test_run_env_command_with_stdin() {
        use crate::course::types::ExecutionLimits;
        use crate::exec::sandbox::{Sandbox, SandboxLevel};

        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        let step = EnvCommand {
            name: "pipe-input".to_string(),
            command: "cat".to_string(),
            args: vec![],
            stdin: Some("hello from stdin".to_string()),
            timeout_seconds: None,
            capture_to: None,
        };
        let result = run_env_command(&sandbox, &step, None, None, 10).unwrap();
        assert_eq!(result.stdout, "hello from stdin");
    }

    #[test]
    fn test_wait_for_service_ready_delay_mode() {
        use crate::course::types::ExecutionLimits;
        use crate::exec::sandbox::{Sandbox, SandboxLevel};

        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        let service = EnvService {
            name: "sleeper".to_string(),
            command: "sleep".to_string(),
            args: vec!["100".to_string()],
            ready_pattern: None,
            ready_stream: None,
            ready_timeout_seconds: 5,
            ready_delay_ms: 50,
            capture_stdout: None,
            capture_stderr: None,
        };
        let mut child = sandbox
            .spawn_service(&service.command, &service.args, None, None)
            .unwrap();
        let result = wait_for_service_ready(&mut child, &service, sandbox.dir());
        assert!(result.is_ok());
        let _ = child.kill();
        let _ = child.wait();
    }

    #[test]
    fn test_wait_for_service_ready_pattern_mode() {
        use crate::course::types::ExecutionLimits;
        use crate::exec::sandbox::{Sandbox, SandboxLevel};

        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        // Write a script that prints a ready message
        sandbox
            .write_file(
                "srv.sh",
                "#!/bin/bash\necho 'server listening on 8080'\nsleep 100\n",
            )
            .unwrap();

        let service = EnvService {
            name: "test-srv".to_string(),
            command: "bash".to_string(),
            args: vec!["srv.sh".to_string()],
            ready_pattern: Some("listening on".to_string()),
            ready_stream: None,
            ready_timeout_seconds: 5,
            ready_delay_ms: 200,
            capture_stdout: None,
            capture_stderr: None,
        };
        let mut child = sandbox
            .spawn_service(&service.command, &service.args, None, None)
            .unwrap();
        let result = wait_for_service_ready(&mut child, &service, sandbox.dir());
        assert!(result.is_ok());
        let _ = child.kill();
        let _ = child.wait();
    }

    #[test]
    fn test_wait_for_service_ready_stderr_pattern() {
        use crate::course::types::ExecutionLimits;
        use crate::exec::sandbox::{Sandbox, SandboxLevel};

        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        // Script that prints ready message to stderr
        sandbox
            .write_file(
                "srv.sh",
                "#!/bin/bash\necho 'server listening on 8080' >&2\nsleep 100\n",
            )
            .unwrap();

        let service = EnvService {
            name: "stderr-srv".to_string(),
            command: "bash".to_string(),
            args: vec!["srv.sh".to_string()],
            ready_pattern: Some("listening on".to_string()),
            ready_stream: Some("stderr".to_string()),
            ready_timeout_seconds: 5,
            ready_delay_ms: 200,
            capture_stdout: None,
            capture_stderr: None,
        };
        let mut child = sandbox
            .spawn_service(&service.command, &service.args, None, None)
            .unwrap();
        let result = wait_for_service_ready(&mut child, &service, sandbox.dir());
        assert!(result.is_ok());
        let _ = child.kill();
        let _ = child.wait();
    }

    #[test]
    fn test_wait_for_service_ready_both_streams() {
        use crate::course::types::ExecutionLimits;
        use crate::exec::sandbox::{Sandbox, SandboxLevel};

        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        // Script that prints ready message to stderr (default "both" should catch it)
        sandbox
            .write_file(
                "srv.sh",
                "#!/bin/bash\necho 'server ready' >&2\nsleep 100\n",
            )
            .unwrap();

        let service = EnvService {
            name: "both-srv".to_string(),
            command: "bash".to_string(),
            args: vec!["srv.sh".to_string()],
            ready_pattern: Some("server ready".to_string()),
            ready_stream: None, // defaults to "both"
            ready_timeout_seconds: 5,
            ready_delay_ms: 200,
            capture_stdout: None,
            capture_stderr: None,
        };
        let mut child = sandbox
            .spawn_service(&service.command, &service.args, None, None)
            .unwrap();
        let result = wait_for_service_ready(&mut child, &service, sandbox.dir());
        assert!(result.is_ok());
        let _ = child.kill();
        let _ = child.wait();
    }

    #[test]
    fn test_wait_for_service_ready_pattern_timeout() {
        use crate::course::types::ExecutionLimits;
        use crate::exec::sandbox::{Sandbox, SandboxLevel};

        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        // Script that never prints the expected pattern
        sandbox
            .write_file("srv.sh", "#!/bin/bash\necho 'starting...'\nsleep 100\n")
            .unwrap();

        let service = EnvService {
            name: "slow-srv".to_string(),
            command: "bash".to_string(),
            args: vec!["srv.sh".to_string()],
            ready_pattern: Some("ready".to_string()),
            ready_stream: None,
            ready_timeout_seconds: 1,
            ready_delay_ms: 200,
            capture_stdout: None,
            capture_stderr: None,
        };
        let mut child = sandbox
            .spawn_service(&service.command, &service.args, None, None)
            .unwrap();
        let result = wait_for_service_ready(&mut child, &service, sandbox.dir());
        assert!(result.is_err());
        let _ = child.kill();
        let _ = child.wait();
    }

    #[test]
    fn test_wait_for_service_delay_mode_crash() {
        use crate::course::types::ExecutionLimits;
        use crate::exec::sandbox::{Sandbox, SandboxLevel};

        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        let service = EnvService {
            name: "crasher".to_string(),
            command: "false".to_string(),
            args: vec![],
            ready_pattern: None,
            ready_stream: None,
            ready_timeout_seconds: 5,
            ready_delay_ms: 100,
            capture_stdout: None,
            capture_stderr: None,
        };
        let mut child = sandbox
            .spawn_service(&service.command, &service.args, None, None)
            .unwrap();
        let result = wait_for_service_ready(&mut child, &service, sandbox.dir());
        assert!(result.is_err());
    }
}
