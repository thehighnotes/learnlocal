use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Studio metadata file stored in each course directory.
const STUDIO_META_FILE: &str = ".learnlocal-studio.json";

/// Returns the studio workspace directory: ~/.local/share/learnlocal/studio/
pub fn workspace_dir() -> anyhow::Result<PathBuf> {
    let data =
        dirs::data_dir().ok_or_else(|| anyhow::anyhow!("Cannot determine data directory"))?;
    let dir = data.join("learnlocal").join("studio");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Studio metadata for a course project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioMeta {
    /// Current project status
    pub status: ProjectStatus,
    /// When the project was created in Studio
    pub created_at: String,
    /// Last modification timestamp
    pub last_modified: String,
    /// Authors who have contributed
    pub authors: Vec<AuthorInfo>,
    /// Change history (audit trail)
    pub history: Vec<HistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Concept,
    Draft,
    Review,
    Published,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorInfo {
    pub name: String,
    pub email: Option<String>,
    pub first_seen: String,
    pub last_seen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: String,
    pub author: String,
    pub action: String,
    pub details: Option<String>,
}

impl StudioMeta {
    pub fn new(author_name: &str) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            status: ProjectStatus::Concept,
            created_at: now.clone(),
            last_modified: now.clone(),
            authors: vec![AuthorInfo {
                name: author_name.to_string(),
                email: None,
                first_seen: now.clone(),
                last_seen: now.clone(),
            }],
            history: vec![HistoryEntry {
                timestamp: now,
                author: author_name.to_string(),
                action: "created".to_string(),
                details: Some("Project created in LearnLocal Studio".to_string()),
            }],
        }
    }

    /// Record an action in the audit trail and update author last_seen.
    pub fn record(&mut self, author_name: &str, action: &str, details: Option<&str>) {
        let now = chrono::Utc::now().to_rfc3339();
        self.last_modified = now.clone();

        // Update or add author
        if let Some(a) = self.authors.iter_mut().find(|a| a.name == author_name) {
            a.last_seen = now.clone();
        } else {
            self.authors.push(AuthorInfo {
                name: author_name.to_string(),
                email: None,
                first_seen: now.clone(),
                last_seen: now.clone(),
            });
        }

        self.history.push(HistoryEntry {
            timestamp: now,
            author: author_name.to_string(),
            action: action.to_string(),
            details: details.map(|s| s.to_string()),
        });
    }
}

/// Load studio metadata from a course directory.
pub fn load_meta(course_path: &Path) -> Option<StudioMeta> {
    let path = course_path.join(STUDIO_META_FILE);
    let contents = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&contents).ok()
}

/// Save studio metadata to a course directory.
pub fn save_meta(course_path: &Path, meta: &StudioMeta) -> anyhow::Result<()> {
    let path = course_path.join(STUDIO_META_FILE);
    let json = serde_json::to_string_pretty(meta)?;
    std::fs::write(&path, json)?;
    Ok(())
}

/// Detect the current author name from git config or environment.
pub fn detect_author() -> String {
    // Try git config
    if let Ok(output) = std::process::Command::new("git")
        .args(["config", "user.name"])
        .output()
    {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !name.is_empty() {
            return name;
        }
    }

    // Try environment
    if let Ok(user) = std::env::var("USER") {
        return user;
    }

    "Unknown Author".to_string()
}

/// A summary of a course project for the welcome screen.
#[derive(Debug, Serialize)]
pub struct ProjectSummary {
    pub path: String,
    pub name: String,
    pub version: String,
    pub status: String,
    pub last_modified: String,
    pub authors: Vec<String>,
    pub lesson_count: usize,
    pub exercise_count: usize,
}

/// List all course projects in the workspace directory.
pub fn list_workspace_projects() -> anyhow::Result<Vec<ProjectSummary>> {
    let ws = workspace_dir()?;
    let mut projects = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&ws) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("course.yaml").exists() {
                if let Ok(summary) = summarize_project(&path) {
                    projects.push(summary);
                }
            }
        }
    }

    // Sort by last_modified descending
    projects.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    Ok(projects)
}

/// List course projects from an arbitrary directory (e.g. courses/).
pub fn list_directory_projects(dir: &Path) -> anyhow::Result<Vec<ProjectSummary>> {
    let mut projects = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("course.yaml").exists() {
                if let Ok(summary) = summarize_project(&path) {
                    projects.push(summary);
                }
            }
        }
    }

    projects.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(projects)
}

fn summarize_project(path: &Path) -> anyhow::Result<ProjectSummary> {
    let info = crate::course::load_course_info(path)?;
    let meta = load_meta(path);

    Ok(ProjectSummary {
        path: path.to_string_lossy().to_string(),
        name: info.name,
        version: info.version,
        status: meta
            .as_ref()
            .map(|m| format!("{:?}", m.status).to_lowercase())
            .unwrap_or_else(|| "published".to_string()),
        last_modified: meta
            .as_ref()
            .map(|m| m.last_modified.clone())
            .unwrap_or_default(),
        authors: meta
            .as_ref()
            .map(|m| m.authors.iter().map(|a| a.name.clone()).collect())
            .unwrap_or_default(),
        lesson_count: info.lesson_count,
        exercise_count: info.total_exercise_count.unwrap_or(0),
    })
}

/// Create a new course concept in the workspace.
pub fn create_concept(
    name: &str,
    language_id: &str,
    language_display: &str,
    extension: &str,
    author_name: &str,
) -> anyhow::Result<PathBuf> {
    let ws = workspace_dir()?;
    let slug = name
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "-")
        .replace("--", "-")
        .trim_matches('-')
        .to_string();

    let project_dir = ws.join(&slug);
    if project_dir.exists() {
        anyhow::bail!("A project with slug '{}' already exists", slug);
    }

    std::fs::create_dir_all(&project_dir)?;
    std::fs::create_dir_all(project_dir.join("lessons"))?;

    // Write course.yaml
    let course_yaml = format!(
        r#"name: "{name}"
version: "0.1.0"
description: ""
author: "{author}"
language:
  id: {lang_id}
  display_name: "{lang_display}"
  extension: "{ext}"
  steps: []
lessons: []
"#,
        name = name,
        author = author_name,
        lang_id = language_id,
        lang_display = language_display,
        ext = extension,
    );
    std::fs::write(project_dir.join("course.yaml"), course_yaml)?;

    // Write studio metadata
    let meta = StudioMeta::new(author_name);
    save_meta(&project_dir, &meta)?;

    Ok(project_dir)
}
