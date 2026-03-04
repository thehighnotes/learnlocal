use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Lightweight course metadata — only reads course.yaml, no lessons/exercises.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CourseInfo {
    pub dir_name: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub language_name: String,
    pub lesson_count: usize,
    pub lesson_ids: Vec<String>,
    pub lesson_titles: Vec<String>,
    pub license: Option<String>,
    pub platform: Option<String>,
    pub estimated_minutes_per_lesson: Option<u32>,
    pub source_dir: PathBuf,
    /// Commands required by language steps (e.g. ["g++"])
    pub step_commands: Vec<String>,
    /// Commands required by environment setup/services/teardown
    pub env_commands: Vec<String>,
    /// Total exercise count (computed from lesson directories)
    pub total_exercise_count: Option<usize>,
    /// How the language toolchain is provisioned
    pub provision: Provision,
}

/// Known platform values for course.yaml platform field.
pub const KNOWN_PLATFORMS: &[&str] = &["linux", "macos", "windows"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: Option<String>,
    pub platform: Option<String>,
    pub language: Language,
    pub lessons: Vec<LessonRef>,
    pub estimated_minutes_per_lesson: Option<u32>,
    /// Populated by the loader, not from YAML
    #[serde(skip)]
    pub loaded_lessons: Vec<Lesson>,
    /// The directory this course was loaded from
    #[serde(skip)]
    pub source_dir: std::path::PathBuf,
}

/// How language toolchains are provisioned for a course.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Provision {
    /// Use system-installed tools (default, current behavior)
    System,
    /// Try system first, fall back to portable download
    Auto,
    /// Use an embedded runtime (e.g. SQLite) compiled into the binary
    Embedded,
    /// User must install manually; show instructions only
    Manual,
}

impl Default for Provision {
    fn default() -> Self {
        Provision::System
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub id: String,
    pub display_name: String,
    pub extension: String,
    pub steps: Vec<ExecutionStep>,
    #[serde(default)]
    pub limits: ExecutionLimits,
    #[serde(default)]
    pub provision: Provision,
    /// For embedded provision: which runtime to use (e.g. "sqlite")
    #[serde(default)]
    pub runtime: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub check_exit_code: bool,
    #[serde(default)]
    pub capture_output: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLimits {
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default = "default_max_output")]
    pub max_output_bytes: usize,
}

impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            timeout_seconds: default_timeout(),
            max_output_bytes: default_max_output(),
        }
    }
}

fn default_timeout() -> u64 {
    10
}

fn default_max_output() -> usize {
    65536
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonRef {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub requires: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub estimated_minutes: Option<u32>,
    pub content: String,
    pub exercises: Vec<String>,
    #[serde(default)]
    pub teaches: Vec<String>,
    pub recap: Option<String>,
    /// Populated by the loader
    #[serde(skip)]
    pub loaded_exercises: Vec<Exercise>,
    /// The raw markdown content loaded from content.md
    #[serde(skip)]
    pub content_markdown: String,
    /// Content split by H2 headers, one section per exercise
    #[serde(skip)]
    pub content_sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Exercise {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub exercise_type: ExerciseType,
    pub prompt: String,
    pub starter: Option<String>,
    #[serde(default)]
    pub files: Vec<ExerciseFile>,
    pub main_file: Option<String>,
    pub input: Option<String>,
    pub validation: Validation,
    #[serde(default)]
    pub hints: Vec<String>,
    pub solution: Option<String>,
    #[serde(default)]
    pub solution_files: Vec<SolutionFile>,
    pub explanation: Option<String>,
    #[serde(default)]
    pub environment: Option<EnvironmentSpec>,
    /// Code golf mode: track character count, show par from solution
    #[serde(default)]
    pub golf: bool,
}

impl Exercise {
    /// Returns ".sh" for Command exercises, the language extension otherwise.
    /// This lets all callers pass `&course.language.extension` without knowing
    /// about the override — the method handles it internally.
    fn effective_extension<'a>(&self, language_ext: &'a str) -> &'a str {
        if self.exercise_type == ExerciseType::Command {
            // Return a static str so lifetime works regardless of language_ext
            ".sh"
        } else {
            language_ext
        }
    }

    /// Get the solution as a list of exercise files suitable for execution.
    /// For single-file exercises, wraps the solution in an ExerciseFile.
    /// For multi-file exercises, merges solution_files with the exercise files.
    pub fn get_solution_files(&self, extension: &str) -> Vec<ExerciseFile> {
        let ext = self.effective_extension(extension);
        if let Some(ref solution) = self.solution {
            let filename = format!("main{}", ext);
            vec![ExerciseFile {
                name: filename,
                editable: true,
                content: solution.clone(),
            }]
        } else if !self.solution_files.is_empty() {
            // Merge solution files with exercise files
            self.files
                .iter()
                .map(|f| {
                    if let Some(sf) = self.solution_files.iter().find(|sf| sf.name == f.name) {
                        ExerciseFile {
                            name: f.name.clone(),
                            editable: f.editable,
                            content: sf.content.clone(),
                        }
                    } else {
                        f.clone()
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }

    /// Get the starter files for this exercise.
    /// For single-file exercises, wraps the starter in an ExerciseFile.
    pub fn get_starter_files(&self, extension: &str) -> Vec<ExerciseFile> {
        let ext = self.effective_extension(extension);
        if let Some(ref starter) = self.starter {
            let filename = format!("main{}", ext);
            vec![ExerciseFile {
                name: filename,
                editable: true,
                content: starter.clone(),
            }]
        } else {
            self.files.clone()
        }
    }

    /// Determine the main file name.
    pub fn get_main_file(&self, extension: &str) -> String {
        let ext = self.effective_extension(extension);
        if let Some(ref mf) = self.main_file {
            mf.clone()
        } else if !self.files.is_empty() {
            self.files
                .iter()
                .find(|f| f.editable)
                .map(|f| f.name.clone())
                .unwrap_or_else(|| self.files[0].name.clone())
        } else {
            format!("main{}", ext)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ExerciseType {
    Write,
    Fix,
    FillBlank,
    MultipleChoice,
    Predict,
    Command,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseFile {
    pub name: String,
    #[serde(default = "default_true")]
    pub editable: bool,
    pub content: String,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionFile {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validation {
    pub method: ValidationMethod,
    pub expected_output: Option<String>,
    pub pattern: Option<String>,
    pub script: Option<String>,
    #[serde(default)]
    pub assertions: Option<Vec<StateAssertion>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationMethod {
    Output,
    CompileOnly,
    Regex,
    Custom,
    State,
}

/// Environment specification for pre-populating a sandbox before exercise execution.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnvironmentSpec {
    #[serde(default)]
    pub files: Vec<EnvFile>,
    #[serde(default)]
    pub dirs: Vec<String>,
    #[serde(default)]
    pub symlinks: Vec<EnvSymlink>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub cwd: Option<String>,
    /// Number of dynamic ports to allocate (injected as LEARNLOCAL_PORT_0, _1, etc.)
    #[serde(default)]
    pub ports: usize,
    /// Commands to run after filesystem setup, before student code.
    #[serde(default)]
    pub setup: Vec<EnvCommand>,
    /// Long-running background services, killed after teardown.
    #[serde(default)]
    pub services: Vec<EnvService>,
    /// Commands to run after student code, before validation.
    #[serde(default)]
    pub teardown: Vec<EnvCommand>,
}

/// A command to run during environment setup or teardown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvCommand {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub stdin: Option<String>,
    pub timeout_seconds: Option<u64>,
    pub capture_to: Option<String>,
}

/// A background service to run during exercise execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvService {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub ready_pattern: Option<String>,
    /// Which stream to watch for ready_pattern: "stdout", "stderr", or "both" (default: "both")
    #[serde(default)]
    pub ready_stream: Option<String>,
    #[serde(default = "default_ready_timeout")]
    pub ready_timeout_seconds: u64,
    #[serde(default = "default_ready_delay")]
    pub ready_delay_ms: u64,
    /// Sandbox-relative path to capture service stdout
    #[serde(default)]
    pub capture_stdout: Option<String>,
    /// Sandbox-relative path to capture service stderr
    #[serde(default)]
    pub capture_stderr: Option<String>,
}

fn default_ready_timeout() -> u64 {
    5
}

fn default_ready_delay() -> u64 {
    200
}

/// A file to create in the sandbox environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvFile {
    pub path: String,
    pub content: String,
    pub permissions: Option<String>,
}

/// A symlink to create in the sandbox environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvSymlink {
    pub link: String,
    pub target: String,
}

/// A single state assertion to check after exercise execution.
#[derive(Debug, Clone, PartialEq)]
pub enum StateAssertion {
    FileExists(String),
    DirExists(String),
    FileNotExists(String),
    DirNotExists(String),
    FileContains(FileContentCheck),
    FileMatches(FilePatternCheck),
    FileEquals(FileContentCheck),
    Permissions(PermissionsCheck),
    Symlink(SymlinkCheck),
    FileCount(FileCountCheck),
    DirEmpty(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileContentCheck {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilePatternCheck {
    pub path: String,
    pub pattern: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionsCheck {
    pub path: String,
    pub mode: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SymlinkCheck {
    pub path: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileCountCheck {
    pub path: String,
    pub count: usize,
}

/// Custom Deserialize for StateAssertion: each YAML entry is a single-key map.
/// Simple variants: `file_exists: path` → `StateAssertion::FileExists(path)`
/// Compound variants: `file_contains: { path: ..., content: ... }` → `StateAssertion::FileContains(...)`
impl<'de> Deserialize<'de> for StateAssertion {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct AssertionHelper {
            file_exists: Option<String>,
            dir_exists: Option<String>,
            file_not_exists: Option<String>,
            dir_not_exists: Option<String>,
            file_contains: Option<FileContentCheck>,
            file_matches: Option<FilePatternCheck>,
            file_equals: Option<FileContentCheck>,
            permissions: Option<PermissionsCheck>,
            symlink: Option<SymlinkCheck>,
            file_count: Option<FileCountCheck>,
            dir_empty: Option<String>,
        }

        let helper = AssertionHelper::deserialize(deserializer)?;

        if let Some(v) = helper.file_exists {
            Ok(StateAssertion::FileExists(v))
        } else if let Some(v) = helper.dir_exists {
            Ok(StateAssertion::DirExists(v))
        } else if let Some(v) = helper.file_not_exists {
            Ok(StateAssertion::FileNotExists(v))
        } else if let Some(v) = helper.dir_not_exists {
            Ok(StateAssertion::DirNotExists(v))
        } else if let Some(v) = helper.file_contains {
            Ok(StateAssertion::FileContains(v))
        } else if let Some(v) = helper.file_matches {
            Ok(StateAssertion::FileMatches(v))
        } else if let Some(v) = helper.file_equals {
            Ok(StateAssertion::FileEquals(v))
        } else if let Some(v) = helper.permissions {
            Ok(StateAssertion::Permissions(v))
        } else if let Some(v) = helper.symlink {
            Ok(StateAssertion::Symlink(v))
        } else if let Some(v) = helper.file_count {
            Ok(StateAssertion::FileCount(v))
        } else if let Some(v) = helper.dir_empty {
            Ok(StateAssertion::DirEmpty(v))
        } else {
            Err(serde::de::Error::custom(
                "unknown assertion type; expected one of: file_exists, dir_exists, \
                 file_not_exists, dir_not_exists, file_contains, file_matches, file_equals, \
                 permissions, symlink, file_count, dir_empty",
            ))
        }
    }
}

impl Serialize for StateAssertion {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            StateAssertion::FileExists(v) => map.serialize_entry("file_exists", v)?,
            StateAssertion::DirExists(v) => map.serialize_entry("dir_exists", v)?,
            StateAssertion::FileNotExists(v) => map.serialize_entry("file_not_exists", v)?,
            StateAssertion::DirNotExists(v) => map.serialize_entry("dir_not_exists", v)?,
            StateAssertion::FileContains(v) => map.serialize_entry("file_contains", v)?,
            StateAssertion::FileMatches(v) => map.serialize_entry("file_matches", v)?,
            StateAssertion::FileEquals(v) => map.serialize_entry("file_equals", v)?,
            StateAssertion::Permissions(v) => map.serialize_entry("permissions", v)?,
            StateAssertion::Symlink(v) => map.serialize_entry("symlink", v)?,
            StateAssertion::FileCount(v) => map.serialize_entry("file_count", v)?,
            StateAssertion::DirEmpty(v) => map.serialize_entry("dir_empty", v)?,
        }
        map.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provision_serde() {
        let yaml = "system";
        let p: Provision = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(p, Provision::System);

        let yaml = "auto";
        let p: Provision = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(p, Provision::Auto);

        let yaml = "embedded";
        let p: Provision = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(p, Provision::Embedded);

        let yaml = "manual";
        let p: Provision = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(p, Provision::Manual);
    }

    #[test]
    fn test_provision_default() {
        assert_eq!(Provision::default(), Provision::System);
    }

    #[test]
    fn test_language_with_provision() {
        let yaml = r#"
id: sql
display_name: "SQL"
extension: ".sql"
steps: []
provision: embedded
runtime: sqlite
"#;
        let lang: Language = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(lang.provision, Provision::Embedded);
        assert_eq!(lang.runtime, Some("sqlite".to_string()));
    }

    #[test]
    fn test_language_without_provision() {
        let yaml = r#"
id: cpp
display_name: "C++"
extension: ".cpp"
steps: []
"#;
        let lang: Language = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(lang.provision, Provision::System);
        assert!(lang.runtime.is_none());
    }

    #[test]
    fn test_exercise_type_serde() {
        let yaml = "write";
        let et: ExerciseType = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(et, ExerciseType::Write);

        let yaml = "fix";
        let et: ExerciseType = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(et, ExerciseType::Fix);

        let yaml = "fill-blank";
        let et: ExerciseType = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(et, ExerciseType::FillBlank);

        let yaml = "multiple-choice";
        let et: ExerciseType = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(et, ExerciseType::MultipleChoice);

        let yaml = "predict";
        let et: ExerciseType = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(et, ExerciseType::Predict);
    }

    #[test]
    fn test_validation_method_serde() {
        let yaml = "output";
        let vm: ValidationMethod = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(vm, ValidationMethod::Output);

        let yaml = "compile-only";
        let vm: ValidationMethod = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(vm, ValidationMethod::CompileOnly);
    }

    #[test]
    fn test_execution_limits_default() {
        let limits = ExecutionLimits::default();
        assert_eq!(limits.timeout_seconds, 10);
        assert_eq!(limits.max_output_bytes, 65536);
    }

    #[test]
    fn test_exercise_file_serde_roundtrip() {
        let yaml = r#"
name: "main.cpp"
editable: true
content: |
  #include <iostream>
  int main() { return 0; }
"#;
        let ef: ExerciseFile = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(ef.name, "main.cpp");
        assert!(ef.editable);
        assert!(ef.content.contains("iostream"));
    }

    #[test]
    fn test_course_yaml_snippet() {
        let yaml = r#"
name: "Test Course"
version: "1.0.0"
description: "A test"
author: "Test"
language:
  id: cpp
  display_name: "C++"
  extension: ".cpp"
  steps:
    - name: compile
      command: "g++"
      args: ["-std=c++17", "-o", "{dir}/{output}", "{dir}/{main}"]
      check_exit_code: true
    - name: run
      command: "{dir}/{output}"
      args: []
      capture_output: true
lessons:
  - id: test
    title: "Test Lesson"
"#;
        let course: Course = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(course.name, "Test Course");
        assert_eq!(course.language.steps.len(), 2);
        assert!(course.language.steps[0].check_exit_code);
        assert!(course.language.steps[1].capture_output);
        assert!(course.platform.is_none());
    }

    #[test]
    fn test_course_yaml_with_platform() {
        let yaml = r#"
name: "Linux Course"
version: "1.0.0"
description: "A test"
author: "Test"
platform: linux
language:
  id: bash
  display_name: "Bash"
  extension: ".sh"
  steps:
    - name: run
      command: "bash"
      args: ["{dir}/{main}"]
      capture_output: true
lessons:
  - id: test
    title: "Test Lesson"
"#;
        let course: Course = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(course.platform, Some("linux".to_string()));
    }

    #[test]
    fn test_course_yaml_without_platform() {
        let yaml = r#"
name: "Any Course"
version: "1.0.0"
description: "A test"
author: "Test"
language:
  id: cpp
  display_name: "C++"
  extension: ".cpp"
  steps:
    - name: compile
      command: "g++"
      args: ["-std=c++17", "-o", "{dir}/{output}", "{dir}/{main}"]
      check_exit_code: true
lessons:
  - id: test
    title: "Test Lesson"
"#;
        let course: Course = serde_yaml::from_str(yaml).unwrap();
        assert!(course.platform.is_none());
    }

    #[test]
    fn test_exercise_yaml_snippet() {
        let yaml = r#"
id: declare
title: "Declare a Variable"
type: write
prompt: "Declare an integer variable named age with the value 25."
starter: |
  #include <iostream>
  int main() {
      // Your code here
      std::cout << age << std::endl;
      return 0;
  }
validation:
  method: output
  expected_output: "25"
hints:
  - "Variables need a type"
  - "The type is int"
solution: |
  #include <iostream>
  int main() {
      int age = 25;
      std::cout << age << std::endl;
      return 0;
  }
explanation: "int age = 25; declares an integer."
"#;
        let ex: Exercise = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(ex.id, "declare");
        assert_eq!(ex.exercise_type, ExerciseType::Write);
        assert_eq!(ex.hints.len(), 2);
        assert!(ex.starter.is_some());
        assert!(ex.solution.is_some());
        assert!(ex.environment.is_none());
    }

    #[test]
    fn test_validation_method_state() {
        let yaml = "state";
        let vm: ValidationMethod = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(vm, ValidationMethod::State);
    }

    #[test]
    fn test_environment_spec_serde() {
        let yaml = "
files:
  - path: data/input.txt
    content: hello world
  - path: script.sh
    content: '#!/bin/bash'
    permissions: '755'
dirs:
  - output
  - output/logs
symlinks:
  - link: latest
    target: data/input.txt
env:
  HOME: /tmp/test
cwd: data
";
        let env: EnvironmentSpec = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(env.files.len(), 2);
        assert_eq!(env.files[0].path, "data/input.txt");
        assert_eq!(env.files[1].permissions, Some("755".to_string()));
        assert_eq!(env.dirs, vec!["output", "output/logs"]);
        assert_eq!(env.symlinks.len(), 1);
        assert_eq!(env.symlinks[0].link, "latest");
        assert_eq!(env.env.get("HOME"), Some(&"/tmp/test".to_string()));
        assert_eq!(env.cwd, Some("data".to_string()));
    }

    #[test]
    fn test_environment_spec_default() {
        let yaml = "{}";
        let env: EnvironmentSpec = serde_yaml::from_str(yaml).unwrap();
        assert!(env.files.is_empty());
        assert!(env.dirs.is_empty());
        assert!(env.symlinks.is_empty());
        assert!(env.env.is_empty());
        assert!(env.cwd.is_none());
        assert_eq!(env.ports, 0);
    }

    #[test]
    fn test_environment_spec_with_ports() {
        let yaml = "ports: 2";
        let env: EnvironmentSpec = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(env.ports, 2);
    }

    #[test]
    fn test_state_assertion_simple_variants() {
        let yaml = "file_exists: output/report.txt";
        let a: StateAssertion = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            a,
            StateAssertion::FileExists("output/report.txt".to_string())
        );

        let yaml = "dir_exists: output/logs";
        let a: StateAssertion = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(a, StateAssertion::DirExists("output/logs".to_string()));

        let yaml = "file_not_exists: temp.txt";
        let a: StateAssertion = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(a, StateAssertion::FileNotExists("temp.txt".to_string()));

        let yaml = "dir_not_exists: tmp";
        let a: StateAssertion = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(a, StateAssertion::DirNotExists("tmp".to_string()));

        let yaml = "dir_empty: output";
        let a: StateAssertion = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(a, StateAssertion::DirEmpty("output".to_string()));
    }

    #[test]
    fn test_state_assertion_compound_variants() {
        let yaml = r#"
file_contains:
  path: output/report.txt
  content: "Total: 55"
"#;
        let a: StateAssertion = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            a,
            StateAssertion::FileContains(FileContentCheck {
                path: "output/report.txt".to_string(),
                content: "Total: 55".to_string(),
            })
        );

        let yaml = r#"
file_matches:
  path: output/log.txt
  pattern: "\\d+ items"
"#;
        let a: StateAssertion = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            a,
            StateAssertion::FileMatches(FilePatternCheck {
                path: "output/log.txt".to_string(),
                pattern: "\\d+ items".to_string(),
            })
        );

        let yaml = r#"
permissions:
  path: script.sh
  mode: "755"
"#;
        let a: StateAssertion = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            a,
            StateAssertion::Permissions(PermissionsCheck {
                path: "script.sh".to_string(),
                mode: "755".to_string(),
            })
        );

        let yaml = r#"
symlink:
  path: latest
  target: data/v2
"#;
        let a: StateAssertion = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            a,
            StateAssertion::Symlink(SymlinkCheck {
                path: "latest".to_string(),
                target: "data/v2".to_string(),
            })
        );

        let yaml = r#"
file_count:
  path: output
  count: 3
"#;
        let a: StateAssertion = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            a,
            StateAssertion::FileCount(FileCountCheck {
                path: "output".to_string(),
                count: 3,
            })
        );
    }

    #[test]
    fn test_state_assertion_list_serde() {
        let yaml = r#"
- file_exists: output/report.txt
- dir_exists: output
- file_contains:
    path: output/report.txt
    content: "Total: 55"
"#;
        let assertions: Vec<StateAssertion> = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(assertions.len(), 3);
        assert_eq!(
            assertions[0],
            StateAssertion::FileExists("output/report.txt".to_string())
        );
        assert_eq!(
            assertions[1],
            StateAssertion::DirExists("output".to_string())
        );
    }

    #[test]
    fn test_validation_with_assertions() {
        let yaml = r#"
method: state
assertions:
  - file_exists: output/report.txt
  - dir_exists: output
"#;
        let v: Validation = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(v.method, ValidationMethod::State);
        assert_eq!(v.assertions.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_exercise_backward_compat_no_environment() {
        // Existing exercises without environment field should still parse
        let yaml = r#"
id: test
title: "Test"
type: write
prompt: "Do something"
starter: "// code"
validation:
  method: output
  expected_output: "42"
hints:
  - "hint"
solution: "answer"
"#;
        let ex: Exercise = serde_yaml::from_str(yaml).unwrap();
        assert!(ex.environment.is_none());
        assert!(ex.validation.assertions.is_none());
    }

    #[test]
    fn test_exercise_with_environment() {
        let yaml = r#"
id: mkdir-test
title: "Create directories"
type: write
prompt: "Create the output directory"
starter: |
  #!/bin/bash
environment:
  files:
    - path: data/input.txt
      content: "hello"
  dirs:
    - work
validation:
  method: state
  assertions:
    - dir_exists: output
    - file_exists: output/result.txt
hints:
  - "Use mkdir"
solution: |
  #!/bin/bash
  mkdir output
  echo "done" > output/result.txt
"#;
        let ex: Exercise = serde_yaml::from_str(yaml).unwrap();
        assert!(ex.environment.is_some());
        let env = ex.environment.unwrap();
        assert_eq!(env.files.len(), 1);
        assert_eq!(env.dirs, vec!["work"]);
        assert_eq!(ex.validation.method, ValidationMethod::State);
        assert_eq!(ex.validation.assertions.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_env_command_serde() {
        let yaml = r#"
name: seed-db
command: sqlite3
args: [app.db]
stdin: |
  CREATE TABLE users(id INTEGER, name TEXT);
timeout_seconds: 5
"#;
        let cmd: EnvCommand = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cmd.name, "seed-db");
        assert_eq!(cmd.command, "sqlite3");
        assert_eq!(cmd.args, vec!["app.db"]);
        assert!(cmd.stdin.is_some());
        assert_eq!(cmd.timeout_seconds, Some(5));
        assert!(cmd.capture_to.is_none());
    }

    #[test]
    fn test_env_command_minimal() {
        let yaml = r#"
name: init
command: git
args: [init]
"#;
        let cmd: EnvCommand = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cmd.name, "init");
        assert_eq!(cmd.command, "git");
        assert!(cmd.stdin.is_none());
        assert!(cmd.timeout_seconds.is_none());
        assert!(cmd.capture_to.is_none());
    }

    #[test]
    fn test_env_command_with_capture() {
        let yaml = r#"
name: dump
command: sqlite3
args: [app.db, ".dump"]
capture_to: db_dump.txt
timeout_seconds: 5
"#;
        let cmd: EnvCommand = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cmd.capture_to, Some("db_dump.txt".to_string()));
    }

    #[test]
    fn test_env_service_serde() {
        let yaml = r#"
name: mock-api
command: python3
args: [server.py]
ready_pattern: "listening on"
ready_timeout_seconds: 10
ready_delay_ms: 500
"#;
        let svc: EnvService = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(svc.name, "mock-api");
        assert_eq!(svc.command, "python3");
        assert_eq!(svc.args, vec!["server.py"]);
        assert_eq!(svc.ready_pattern, Some("listening on".to_string()));
        assert_eq!(svc.ready_timeout_seconds, 10);
        assert_eq!(svc.ready_delay_ms, 500);
        assert!(svc.capture_stdout.is_none());
        assert!(svc.capture_stderr.is_none());
    }

    #[test]
    fn test_env_service_with_capture() {
        let yaml = r#"
name: api-server
command: python3
args: [server.py]
capture_stdout: server_stdout.log
capture_stderr: server_stderr.log
"#;
        let svc: EnvService = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(svc.capture_stdout, Some("server_stdout.log".to_string()));
        assert_eq!(svc.capture_stderr, Some("server_stderr.log".to_string()));
    }

    #[test]
    fn test_env_service_defaults() {
        let yaml = r#"
name: server
command: node
args: [app.js]
"#;
        let svc: EnvService = serde_yaml::from_str(yaml).unwrap();
        assert!(svc.ready_pattern.is_none());
        assert_eq!(svc.ready_timeout_seconds, 5);
        assert_eq!(svc.ready_delay_ms, 200);
    }

    #[test]
    fn test_environment_spec_with_setup_services_teardown() {
        let yaml = r#"
files:
  - path: server.py
    content: "print('listening on 8080')"
dirs:
  - data
setup:
  - name: init-repo
    command: git
    args: [init]
services:
  - name: mock-api
    command: python3
    args: [server.py]
    ready_pattern: "listening on"
teardown:
  - name: dump
    command: echo
    args: [done]
    capture_to: result.txt
"#;
        let env: EnvironmentSpec = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(env.setup.len(), 1);
        assert_eq!(env.setup[0].name, "init-repo");
        assert_eq!(env.services.len(), 1);
        assert_eq!(env.services[0].name, "mock-api");
        assert_eq!(env.teardown.len(), 1);
        assert_eq!(env.teardown[0].capture_to, Some("result.txt".to_string()));
    }

    #[test]
    fn test_environment_spec_backward_compat_no_new_fields() {
        // Existing YAML without setup/services/teardown still parses fine
        let yaml = r#"
files:
  - path: data/input.txt
    content: hello
dirs:
  - output
"#;
        let env: EnvironmentSpec = serde_yaml::from_str(yaml).unwrap();
        assert!(env.setup.is_empty());
        assert!(env.services.is_empty());
        assert!(env.teardown.is_empty());
    }

    #[test]
    fn test_exercise_type_command_serde() {
        let yaml = "command";
        let et: ExerciseType = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(et, ExerciseType::Command);

        // Roundtrip
        let serialized = serde_yaml::to_string(&et).unwrap();
        assert!(serialized.contains("command"));
    }

    #[test]
    fn test_command_exercise_uses_sh_extension() {
        // A command exercise in a C++ course should use .sh, not .cpp
        let exercise = Exercise {
            id: "git-init".to_string(),
            title: "Init Repo".to_string(),
            exercise_type: ExerciseType::Command,
            prompt: "Initialize a git repo".to_string(),
            starter: Some("# your commands here".to_string()),
            files: vec![],
            main_file: None,
            input: None,
            validation: Validation {
                method: ValidationMethod::State,
                expected_output: None,
                pattern: None,
                script: None,
                assertions: Some(vec![StateAssertion::DirExists(".git".to_string())]),
            },
            hints: vec!["Use git init".to_string()],
            solution: Some("git init".to_string()),
            solution_files: vec![],
            explanation: None,
            environment: None,
            golf: false,
        };

        // Even though we pass ".cpp", command exercises use ".sh"
        assert_eq!(exercise.get_main_file(".cpp"), "main.sh");

        let starter_files = exercise.get_starter_files(".cpp");
        assert_eq!(starter_files.len(), 1);
        assert_eq!(starter_files[0].name, "main.sh");

        let solution_files = exercise.get_solution_files(".cpp");
        assert_eq!(solution_files.len(), 1);
        assert_eq!(solution_files[0].name, "main.sh");
    }

    #[test]
    fn test_non_command_exercise_uses_language_extension() {
        let exercise = Exercise {
            id: "hello".to_string(),
            title: "Hello".to_string(),
            exercise_type: ExerciseType::Write,
            prompt: "Write hello world".to_string(),
            starter: Some("// code".to_string()),
            files: vec![],
            main_file: None,
            input: None,
            validation: Validation {
                method: ValidationMethod::Output,
                expected_output: Some("hello".to_string()),
                pattern: None,
                script: None,
                assertions: None,
            },
            hints: vec!["hint".to_string()],
            solution: Some("answer".to_string()),
            solution_files: vec![],
            explanation: None,
            environment: None,
            golf: false,
        };

        // Write exercises use the language extension as normal
        assert_eq!(exercise.get_main_file(".cpp"), "main.cpp");
        assert_eq!(exercise.get_starter_files(".cpp")[0].name, "main.cpp");
        assert_eq!(exercise.get_solution_files(".cpp")[0].name, "main.cpp");
    }

    #[test]
    fn test_command_exercise_yaml_roundtrip() {
        let yaml = r#"
id: git-init
title: "Initialize a Repository"
type: command
prompt: |
  Initialize a git repository.
starter: |
  # Your commands here
validation:
  method: state
  assertions:
    - dir_exists: .git
hints:
  - "Use git init"
solution: |
  git init
"#;
        let ex: Exercise = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(ex.exercise_type, ExerciseType::Command);
        assert_eq!(ex.id, "git-init");
        assert!(ex.starter.is_some());
    }
}
