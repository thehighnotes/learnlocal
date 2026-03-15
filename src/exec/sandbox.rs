use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::TempDir;

use crate::config::SandboxLevelPref;
use crate::course::types::ExecutionLimits;
use crate::error::{LearnLocalError, Result};

#[derive(Debug, Default)]
pub struct StepOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SandboxLevel {
    Basic,
    Firejail,
    Bubblewrap,
}

impl SandboxLevel {
    /// Detect the best available sandbox level, respecting user preference.
    pub fn detect(pref: &SandboxLevelPref) -> Self {
        match pref {
            SandboxLevelPref::Basic => SandboxLevel::Basic,
            SandboxLevelPref::Contained => {
                if which_exists("firejail") {
                    SandboxLevel::Firejail
                } else if which_exists("bwrap") {
                    SandboxLevel::Bubblewrap
                } else {
                    // Prefer basic rather than failing — user asked for contained but it's not available
                    SandboxLevel::Basic
                }
            }
            SandboxLevelPref::Auto => {
                if which_exists("firejail") {
                    SandboxLevel::Firejail
                } else if which_exists("bwrap") {
                    SandboxLevel::Bubblewrap
                } else {
                    SandboxLevel::Basic
                }
            }
        }
    }
}

fn which_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub struct Sandbox {
    temp_dir: TempDir,
    limits: ExecutionLimits,
    level: SandboxLevel,
}

impl Sandbox {
    pub fn new(limits: &ExecutionLimits, level: SandboxLevel) -> Result<Self> {
        let temp_dir = TempDir::new()
            .map_err(|e| LearnLocalError::Execution(format!("Failed to create temp dir: {}", e)))?;
        Ok(Self {
            temp_dir,
            limits: limits.clone(),
            level,
        })
    }

    pub fn dir(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn write_file(&self, name: &str, content: &str) -> Result<PathBuf> {
        let path = self.temp_dir.path().join(name);
        // Verify the resolved path is still within the sandbox
        let canonical_base = self
            .temp_dir
            .path()
            .canonicalize()
            .unwrap_or_else(|_| self.temp_dir.path().to_path_buf());
        // Create parent dirs if needed (for nested file structures)
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let canonical_path = path.canonicalize().unwrap_or_else(|_| {
            // File doesn't exist yet, canonicalize parent and append filename
            path.parent()
                .and_then(|p| p.canonicalize().ok())
                .map(|p| p.join(path.file_name().unwrap_or_default()))
                .unwrap_or_else(|| path.clone())
        });
        if !canonical_path.starts_with(&canonical_base) {
            return Err(LearnLocalError::Execution(format!(
                "Path traversal blocked: '{}' escapes sandbox",
                name
            )));
        }
        std::fs::write(&path, content)?;
        Ok(path)
    }

    pub fn run_command(
        &self,
        command: &str,
        args: &[String],
        stdin_input: Option<&str>,
        env_vars: Option<&std::collections::HashMap<String, String>>,
        cwd_override: Option<&Path>,
    ) -> Result<StepOutput> {
        self.run_command_inner(
            command,
            args,
            stdin_input,
            env_vars,
            cwd_override,
            self.limits.timeout_seconds,
            false,
        )
    }

    /// Like `run_command` but with loopback networking enabled for sandbox.
    /// Used when an exercise defines background services that student code needs to reach.
    pub fn run_command_with_loopback(
        &self,
        command: &str,
        args: &[String],
        stdin_input: Option<&str>,
        env_vars: Option<&std::collections::HashMap<String, String>>,
        cwd_override: Option<&Path>,
        allow_loopback: bool,
    ) -> Result<StepOutput> {
        self.run_command_inner(
            command,
            args,
            stdin_input,
            env_vars,
            cwd_override,
            self.limits.timeout_seconds,
            allow_loopback,
        )
    }

    /// Like `run_command` but with a caller-specified timeout. Used by setup/teardown steps.
    pub fn run_command_with_timeout(
        &self,
        command: &str,
        args: &[String],
        stdin_input: Option<&str>,
        env_vars: Option<&std::collections::HashMap<String, String>>,
        cwd_override: Option<&Path>,
        timeout_seconds: u64,
    ) -> Result<StepOutput> {
        self.run_command_inner(
            command,
            args,
            stdin_input,
            env_vars,
            cwd_override,
            timeout_seconds,
            false,
        )
    }

    /// Spawn a background service process. Does NOT wait for exit.
    /// Returns the Child handle for the caller to manage.
    pub fn spawn_service(
        &self,
        command: &str,
        args: &[String],
        env_vars: Option<&std::collections::HashMap<String, String>>,
        cwd_override: Option<&Path>,
    ) -> Result<std::process::Child> {
        let (actual_cmd, actual_args) = self.wrap_command(command, args, true);

        let mut cmd = Command::new(&actual_cmd);
        cmd.args(&actual_args)
            .current_dir(cwd_override.unwrap_or(self.temp_dir.path()))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        if let Some(vars) = env_vars {
            cmd.envs(vars);
        }

        cmd.spawn().map_err(|e| {
            let mut msg = format!("Failed to spawn service '{}': {}", actual_cmd, e);
            if e.kind() == std::io::ErrorKind::NotFound {
                if let Some(hint) = crate::exec::toolcheck::suggest_install(command) {
                    msg.push_str(&format!("\n\nTo install: {}", hint));
                }
            }
            LearnLocalError::Execution(msg)
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn run_command_inner(
        &self,
        command: &str,
        args: &[String],
        stdin_input: Option<&str>,
        env_vars: Option<&std::collections::HashMap<String, String>>,
        cwd_override: Option<&Path>,
        timeout_seconds: u64,
        allow_loopback: bool,
    ) -> Result<StepOutput> {
        let (actual_cmd, actual_args) = self.wrap_command(command, args, allow_loopback);

        let mut cmd = Command::new(&actual_cmd);
        cmd.args(&actual_args)
            .current_dir(cwd_override.unwrap_or(self.temp_dir.path()))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(vars) = env_vars {
            cmd.envs(vars);
        }

        if stdin_input.is_some() {
            cmd.stdin(Stdio::piped());
        } else {
            cmd.stdin(Stdio::null());
        }

        let mut child = cmd.spawn().map_err(|e| {
            let mut msg = format!("Failed to spawn '{}': {}", actual_cmd, e);
            if e.kind() == std::io::ErrorKind::NotFound {
                if let Some(hint) = crate::exec::toolcheck::suggest_install(command) {
                    msg.push_str(&format!("\n\nTo install: {}", hint));
                }
            }
            LearnLocalError::Execution(msg)
        })?;

        if let Some(input) = stdin_input {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(input.as_bytes());
            }
        }

        let timeout = Duration::from_secs(timeout_seconds);
        let child_id = child.id();

        let timed_out = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let timed_out_flag = timed_out.clone();
        let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let cancel_flag = cancel.clone();
        let timer = std::thread::spawn(move || {
            // Sleep in small increments so we can check cancellation
            let deadline = std::time::Instant::now() + timeout;
            while std::time::Instant::now() < deadline {
                if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
                    return; // Process finished normally, don't kill anything
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            if !cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
                timed_out_flag.store(true, std::sync::atomic::Ordering::SeqCst);
                let _ = Command::new("kill")
                    .args(["-9", &child_id.to_string()])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
        });

        match child.wait_with_output() {
            Ok(output) => {
                cancel.store(true, std::sync::atomic::Ordering::SeqCst);
                drop(timer);

                let stdout = truncate_output(
                    String::from_utf8_lossy(&output.stdout).to_string(),
                    self.limits.max_output_bytes,
                );
                let stderr = truncate_output(
                    String::from_utf8_lossy(&output.stderr).to_string(),
                    self.limits.max_output_bytes,
                );

                let did_timeout = timed_out.load(std::sync::atomic::Ordering::SeqCst);

                Ok(StepOutput {
                    stdout,
                    stderr,
                    exit_code: output.status.code().unwrap_or(-1),
                    timed_out: did_timeout,
                })
            }
            Err(e) => Err(LearnLocalError::Execution(format!(
                "Failed to wait for process: {}",
                e
            ))),
        }
    }

    fn wrap_command(
        &self,
        command: &str,
        args: &[String],
        allow_loopback: bool,
    ) -> (String, Vec<String>) {
        let tmpdir = self.temp_dir.path().to_string_lossy().to_string();

        match self.level {
            SandboxLevel::Basic => (command.to_string(), args.to_vec()),
            SandboxLevel::Firejail => {
                let net_flag = if allow_loopback {
                    "--net=lo"
                } else {
                    "--net=none"
                };
                let mut firejail_args = vec![
                    format!("--whitelist={}", tmpdir),
                    "--quiet".to_string(),
                    net_flag.to_string(),
                    "--".to_string(),
                    command.to_string(),
                ];
                firejail_args.extend(args.iter().cloned());
                ("firejail".to_string(), firejail_args)
            }
            SandboxLevel::Bubblewrap => {
                let mut bwrap_args = vec![
                    "--ro-bind".to_string(),
                    "/".to_string(),
                    "/".to_string(),
                    "--bind".to_string(),
                    tmpdir.clone(),
                    tmpdir,
                ];
                if !allow_loopback {
                    bwrap_args.push("--unshare-net".to_string());
                }
                bwrap_args.push("--die-with-parent".to_string());
                bwrap_args.push("--".to_string());
                bwrap_args.push(command.to_string());
                bwrap_args.extend(args.iter().cloned());
                ("bwrap".to_string(), bwrap_args)
            }
        }
    }
}

fn truncate_output(s: String, max_bytes: usize) -> String {
    if s.len() > max_bytes {
        // Find the last valid UTF-8 char boundary at or before max_bytes
        let boundary = (0..=max_bytes)
            .rev()
            .find(|&i| s.is_char_boundary(i))
            .unwrap_or(0);
        let mut truncated = s[..boundary].to_string();
        truncated.push_str("\n... (output truncated)");
        truncated
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_write_and_read() {
        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        sandbox.write_file("test.txt", "hello world").unwrap();
        let content = std::fs::read_to_string(sandbox.dir().join("test.txt")).unwrap();
        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_sandbox_run_echo() {
        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        let result = sandbox
            .run_command("echo", &["hello".to_string()], None, None, None)
            .unwrap();
        assert_eq!(result.stdout.trim(), "hello");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_sandbox_run_with_stdin() {
        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        let result = sandbox
            .run_command("cat", &[], Some("piped input"), None, None)
            .unwrap();
        assert_eq!(result.stdout, "piped input");
    }

    #[test]
    fn test_sandbox_nonzero_exit() {
        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        let result = sandbox.run_command("false", &[], None, None, None).unwrap();
        assert_ne!(result.exit_code, 0);
    }

    #[test]
    fn test_truncate_output() {
        let long = "a".repeat(100);
        let truncated = truncate_output(long, 50);
        assert!(truncated.len() < 100);
        assert!(truncated.contains("truncated"));
    }

    #[test]
    fn test_which_exists_echo() {
        assert!(which_exists("echo"));
    }

    #[test]
    fn test_which_exists_nonexistent() {
        assert!(!which_exists("definitely_not_a_real_command_12345"));
    }

    #[test]
    fn test_sandbox_level_detect_basic() {
        let level = SandboxLevel::detect(&SandboxLevelPref::Basic);
        assert_eq!(level, SandboxLevel::Basic);
    }

    #[test]
    fn test_sandbox_timeout_kills_process() {
        let limits = ExecutionLimits {
            timeout_seconds: 1,
            max_output_bytes: 65536,
        };
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        let result = sandbox
            .run_command("sleep", &["100".to_string()], None, None, None)
            .unwrap();
        assert!(result.timed_out);
    }

    #[test]
    fn test_wrap_command_basic() {
        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        let (cmd, args) =
            sandbox.wrap_command("g++", &["-o".to_string(), "out".to_string()], false);
        assert_eq!(cmd, "g++");
        assert_eq!(args, vec!["-o", "out"]);
    }

    #[test]
    fn test_wrap_command_firejail() {
        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Firejail).unwrap();
        let (cmd, args) =
            sandbox.wrap_command("g++", &["-o".to_string(), "out".to_string()], false);
        assert_eq!(cmd, "firejail");
        assert!(args.contains(&"--quiet".to_string()));
        assert!(args.contains(&"--net=none".to_string()));
        assert!(args.contains(&"g++".to_string()));
    }

    #[test]
    fn test_wrap_command_firejail_loopback() {
        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Firejail).unwrap();
        let (cmd, args) = sandbox.wrap_command("python3", &["server.py".to_string()], true);
        assert_eq!(cmd, "firejail");
        assert!(args.contains(&"--net=lo".to_string()));
        assert!(!args.contains(&"--net=none".to_string()));
    }

    #[test]
    fn test_wrap_command_bubblewrap() {
        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Bubblewrap).unwrap();
        let (cmd, args) =
            sandbox.wrap_command("g++", &["-o".to_string(), "out".to_string()], false);
        assert_eq!(cmd, "bwrap");
        assert!(args.contains(&"--unshare-net".to_string()));
        assert!(args.contains(&"--die-with-parent".to_string()));
        assert!(args.contains(&"g++".to_string()));
    }

    #[test]
    fn test_wrap_command_bubblewrap_loopback() {
        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Bubblewrap).unwrap();
        let (cmd, args) = sandbox.wrap_command("node", &["app.js".to_string()], true);
        assert_eq!(cmd, "bwrap");
        assert!(!args.contains(&"--unshare-net".to_string()));
        assert!(args.contains(&"--die-with-parent".to_string()));
    }

    #[test]
    fn test_spawn_service_and_kill() {
        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        let mut child = sandbox
            .spawn_service("sleep", &["100".to_string()], None, None)
            .unwrap();
        // Process should be running
        assert!(child.try_wait().unwrap().is_none());
        // Kill it
        let _ = child.kill();
        let _ = child.wait();
    }

    #[test]
    fn test_run_command_with_timeout() {
        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        let result = sandbox
            .run_command_with_timeout("echo", &["hi".to_string()], None, None, None, 5)
            .unwrap();
        assert_eq!(result.stdout.trim(), "hi");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_run_command_with_loopback() {
        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        let result = sandbox
            .run_command_with_loopback("echo", &["loopback".to_string()], None, None, None, true)
            .unwrap();
        assert_eq!(result.stdout.trim(), "loopback");
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_run_command_with_env_vars() {
        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        let mut vars = std::collections::HashMap::new();
        vars.insert("MY_VAR".to_string(), "hello_env".to_string());
        let result = sandbox
            .run_command(
                "sh",
                &["-c".to_string(), "echo $MY_VAR".to_string()],
                None,
                Some(&vars),
                None,
            )
            .unwrap();
        assert_eq!(result.stdout.trim(), "hello_env");
    }

    #[test]
    fn test_run_command_with_cwd_override() {
        let limits = ExecutionLimits::default();
        let sandbox = Sandbox::new(&limits, SandboxLevel::Basic).unwrap();
        let subdir = sandbox.dir().join("mydir");
        std::fs::create_dir(&subdir).unwrap();
        let result = sandbox
            .run_command("pwd", &[], None, None, Some(&subdir))
            .unwrap();
        assert!(result.stdout.trim().ends_with("mydir"));
    }
}
