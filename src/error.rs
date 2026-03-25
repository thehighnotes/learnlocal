use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum LearnLocalError {
    #[error("Course load error: {0}")]
    CourseLoad(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("Progress error: {0}")]
    Progress(String),

    #[error("Editor error: {0}")]
    Editor(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Semver error: {0}")]
    Semver(#[from] semver::Error),

    #[error("Community error: {0}")]
    Community(String),
}

pub type Result<T> = std::result::Result<T, LearnLocalError>;
