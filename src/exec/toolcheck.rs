use crate::course::types::{EnvironmentSpec, Language, Provision};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ToolStatus {
    pub command: String,
    pub found: bool,
    pub install_hint: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PlatformStatus {
    pub required: Option<String>,
    pub current: String,
    pub supported: bool,
}

/// Check whether the current platform matches the course requirement.
pub fn check_platform(required: &Option<String>) -> PlatformStatus {
    let current = std::env::consts::OS.to_string();
    let supported = match required {
        None => true,
        Some(req) => req == &current,
    };
    PlatformStatus {
        required: required.clone(),
        current,
        supported,
    }
}

/// Extract the base command from a step command string.
/// Skips placeholders like `{dir}/{output}` which are compiled binaries.
pub fn extract_command(step_cmd: &str) -> Option<String> {
    let first_word = step_cmd.split_whitespace().next()?;
    // If it contains {placeholder}, it's a compiled binary path, not a tool
    if first_word.contains('{') {
        return None;
    }
    Some(first_word.to_string())
}

/// Check whether a command exists on the system PATH.
pub fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Suggest an install command for known language tools.
pub fn suggest_install(cmd: &str) -> Option<String> {
    match cmd {
        "g++" | "gcc" => Some("sudo apt install g++  (or: brew install gcc)".to_string()),
        "clang++" | "clang" => Some("sudo apt install clang  (or: brew install llvm)".to_string()),
        "python3" | "python" => {
            Some("sudo apt install python3  (or: brew install python)".to_string())
        }
        "rustc" | "cargo" => {
            Some("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh".to_string())
        }
        "node" | "nodejs" => Some("sudo apt install nodejs  (or: brew install node)".to_string()),
        "javac" | "java" => {
            Some("sudo apt install default-jdk  (or: brew install openjdk)".to_string())
        }
        "go" => Some("sudo apt install golang  (or: brew install go)".to_string()),
        "ruby" => Some("sudo apt install ruby  (or: brew install ruby)".to_string()),
        "bash" => Some("sudo apt install bash  (or: brew install bash)".to_string()),
        _ => None,
    }
}

/// Check all language tools required by a course's execution steps.
/// For embedded provision, returns empty (all tools are built-in).
pub fn check_language_tools(language: &Language) -> Vec<ToolStatus> {
    // Embedded provision has no external tool requirements
    if language.provision == Provision::Embedded {
        return vec![];
    }

    let mut results = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for step in &language.steps {
        if let Some(cmd) = extract_command(&step.command) {
            if seen.insert(cmd.clone()) {
                let found = command_exists(&cmd);
                let install_hint = if !found { suggest_install(&cmd) } else { None };
                results.push(ToolStatus {
                    command: cmd,
                    found,
                    install_hint,
                });
            }
        }
    }

    results
}

/// Extract unique command names from language steps (for CourseInfo).
pub fn extract_step_commands(language: &Language) -> Vec<String> {
    let mut cmds = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for step in &language.steps {
        if let Some(cmd) = extract_command(&step.command) {
            if seen.insert(cmd.clone()) {
                cmds.push(cmd);
            }
        }
    }
    cmds
}

/// Extract unique command names from environment setup/services/teardown steps.
/// Used by home screen tool check when courses with env commands are loaded.
pub fn extract_env_commands(env: &EnvironmentSpec) -> Vec<String> {
    let mut cmds = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for step in &env.setup {
        if seen.insert(step.command.clone()) {
            cmds.push(step.command.clone());
        }
    }
    for svc in &env.services {
        if seen.insert(svc.command.clone()) {
            cmds.push(svc.command.clone());
        }
    }
    for step in &env.teardown {
        if seen.insert(step.command.clone()) {
            cmds.push(step.command.clone());
        }
    }
    cmds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_command_normal() {
        assert_eq!(extract_command("g++"), Some("g++".to_string()));
        assert_eq!(
            extract_command("python3 {dir}/{main}"),
            Some("python3".to_string())
        );
    }

    #[test]
    fn test_extract_command_placeholder() {
        // Compiled binary placeholder should return None
        assert_eq!(extract_command("{dir}/{output}"), None);
        assert_eq!(extract_command("{dir}/main"), None);
    }

    #[test]
    fn test_extract_command_empty() {
        assert_eq!(extract_command(""), None);
    }

    #[test]
    fn test_suggest_install_known() {
        assert!(suggest_install("g++").is_some());
        assert!(suggest_install("python3").is_some());
        assert!(suggest_install("rustc").is_some());
        assert!(suggest_install("node").is_some());
        assert!(suggest_install("javac").is_some());
        assert!(suggest_install("go").is_some());
    }

    #[test]
    fn test_suggest_install_unknown() {
        assert!(suggest_install("some_obscure_tool").is_none());
    }

    #[test]
    fn test_command_exists_echo() {
        // echo should exist on all POSIX systems
        assert!(command_exists("echo"));
    }

    #[test]
    fn test_command_exists_nonexistent() {
        assert!(!command_exists("nonexistent_cmd_xyz_12345"));
    }

    #[test]
    fn test_check_platform_none_supported() {
        let status = check_platform(&None);
        assert!(status.supported);
        assert!(status.required.is_none());
    }

    #[test]
    fn test_check_platform_current_os_supported() {
        let current = std::env::consts::OS.to_string();
        let status = check_platform(&Some(current.clone()));
        assert!(status.supported);
        assert_eq!(status.current, current);
    }

    #[test]
    fn test_check_platform_other_os_not_supported() {
        let status = check_platform(&Some("fakeos".to_string()));
        assert!(!status.supported);
        assert_eq!(status.required, Some("fakeos".to_string()));
    }

    #[test]
    fn test_suggest_install_bash() {
        assert!(suggest_install("bash").is_some());
    }

    #[test]
    fn test_extract_env_commands() {
        use crate::course::types::{EnvCommand, EnvService};
        let env = EnvironmentSpec {
            setup: vec![
                EnvCommand {
                    name: "init".to_string(),
                    command: "git".to_string(),
                    args: vec![],
                    stdin: None,
                    timeout_seconds: None,
                    capture_to: None,
                },
                EnvCommand {
                    name: "seed".to_string(),
                    command: "sqlite3".to_string(),
                    args: vec![],
                    stdin: None,
                    timeout_seconds: None,
                    capture_to: None,
                },
            ],
            services: vec![EnvService {
                name: "server".to_string(),
                command: "python3".to_string(),
                args: vec![],
                ready_pattern: None,
                ready_stream: None,
                ready_timeout_seconds: 5,
                ready_delay_ms: 200,
                capture_stdout: None,
                capture_stderr: None,
            }],
            teardown: vec![EnvCommand {
                name: "dump".to_string(),
                command: "sqlite3".to_string(), // duplicate — should be deduped
                args: vec![],
                stdin: None,
                timeout_seconds: None,
                capture_to: None,
            }],
            ..Default::default()
        };
        let cmds = extract_env_commands(&env);
        assert_eq!(cmds, vec!["git", "sqlite3", "python3"]);
    }

    #[test]
    fn test_extract_env_commands_empty() {
        let env = EnvironmentSpec::default();
        assert!(extract_env_commands(&env).is_empty());
    }
}
