use serde::{Deserialize, Serialize};

/// The top-level registry index (registry.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub version: u32,
    pub updated_at: String,
    pub courses: Vec<RegistryCourse>,
}

/// A single course entry in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryCourse {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    #[serde(default)]
    pub author_github: Option<String>,
    pub description: String,
    pub language_id: String,
    pub language_display: String,
    #[serde(default)]
    pub license: Option<String>,
    pub lessons: usize,
    pub exercises: usize,
    #[serde(default)]
    pub has_stages: bool,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default = "default_provision")]
    pub provision: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub estimated_hours: Option<f32>,
    pub download_url: String,
    /// Format: "sha256:<hex digest>"
    pub checksum: String,
    #[serde(default)]
    pub published_at: String,
    #[serde(default)]
    pub min_learnlocal_version: Option<String>,
    // Server-provided fields (not present in static registry)
    #[serde(default)]
    pub avg_rating: Option<f64>,
    #[serde(default)]
    pub review_count: Option<u32>,
    #[serde(default)]
    pub downloads: Option<u64>,
    // Provenance
    #[serde(default)]
    pub owner_github: Option<String>,
    #[serde(default)]
    pub forked_from: Option<ForkInfo>,
}

/// Fork lineage — tracks where a course was derived from.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkInfo {
    pub id: String,
    pub version: Option<String>,
    pub author: Option<String>,
}

fn default_provision() -> String {
    "system".to_string()
}

/// How the registry data was obtained.
#[derive(Debug, Clone)]
pub enum RegistrySource {
    /// Freshly fetched from remote URL.
    Remote,
    /// Loaded from local cache.
    Cached { age_secs: u64 },
    /// No data available (empty fallback).
    Empty,
}

impl std::fmt::Display for RegistrySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistrySource::Remote => write!(f, "Remote"),
            RegistrySource::Cached { age_secs } => {
                let hours = age_secs / 3600;
                if hours < 1 {
                    write!(f, "Cached (< 1h ago)")
                } else if hours < 24 {
                    write!(f, "Cached ({}h ago)", hours)
                } else {
                    write!(f, "Cached ({}d ago)", hours / 24)
                }
            }
            RegistrySource::Empty => write!(f, "Offline"),
        }
    }
}

/// Result of fetching the registry.
pub struct RegistryResult {
    pub registry: Registry,
    pub source: RegistrySource,
}

/// Community-related configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommunityConfig {
    #[serde(default = "default_registry_url")]
    pub registry_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
}

impl Default for CommunityConfig {
    fn default() -> Self {
        Self {
            registry_url: default_registry_url(),
            auth_token: None,
        }
    }
}

fn default_registry_url() -> String {
    "https://learnlocal.aiquest.info/api/v1/courses".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_deserialize() {
        let json = r#"{
            "version": 1,
            "updated_at": "2026-03-22T00:00:00Z",
            "courses": [
                {
                    "id": "cpp-fundamentals",
                    "name": "C++ Fundamentals",
                    "version": "2.0.0",
                    "author": "LearnLocal Community",
                    "description": "Learn C++ from scratch.",
                    "language_id": "cpp",
                    "language_display": "C++",
                    "lessons": 8,
                    "exercises": 58,
                    "download_url": "https://example.com/cpp.tar.gz",
                    "checksum": "sha256:abc123",
                    "tags": ["beginner", "cpp"]
                }
            ]
        }"#;
        let registry: Registry = serde_json::from_str(json).unwrap();
        assert_eq!(registry.version, 1);
        assert_eq!(registry.courses.len(), 1);
        assert_eq!(registry.courses[0].id, "cpp-fundamentals");
        assert_eq!(registry.courses[0].lessons, 8);
        assert_eq!(registry.courses[0].provision, "system");
        assert!(!registry.courses[0].has_stages);
        assert!(registry.courses[0].platform.is_none());
    }

    #[test]
    fn test_registry_deserialize_all_fields() {
        let json = r#"{
            "version": 1,
            "updated_at": "2026-03-22T00:00:00Z",
            "courses": [
                {
                    "id": "python-fundamentals",
                    "name": "Python Fundamentals",
                    "version": "1.1.0",
                    "author": "LearnLocal Community",
                    "author_github": "thehighnotes",
                    "description": "Learn Python.",
                    "language_id": "python3",
                    "language_display": "Python",
                    "license": "CC-BY-4.0",
                    "lessons": 8,
                    "exercises": 57,
                    "has_stages": true,
                    "platform": "linux",
                    "provision": "auto",
                    "tags": ["beginner", "python"],
                    "estimated_hours": 4.5,
                    "download_url": "https://example.com/python.tar.gz",
                    "checksum": "sha256:def456",
                    "published_at": "2026-03-22T00:00:00Z",
                    "min_learnlocal_version": "0.2.0"
                }
            ]
        }"#;
        let registry: Registry = serde_json::from_str(json).unwrap();
        let c = &registry.courses[0];
        assert_eq!(c.author_github.as_deref(), Some("thehighnotes"));
        assert_eq!(c.license.as_deref(), Some("CC-BY-4.0"));
        assert!(c.has_stages);
        assert_eq!(c.platform.as_deref(), Some("linux"));
        assert_eq!(c.provision, "auto");
        assert_eq!(c.estimated_hours, Some(4.5));
        assert_eq!(c.min_learnlocal_version.as_deref(), Some("0.2.0"));
    }

    #[test]
    fn test_community_config_default() {
        let config = CommunityConfig::default();
        assert!(config.registry_url.contains("learnlocal"));
    }

    #[test]
    fn test_community_config_deserialize_empty() {
        let yaml = "{}";
        let config: CommunityConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.registry_url.contains("learnlocal"));
    }

    #[test]
    fn test_community_config_custom_url() {
        let yaml = r#"registry_url: "https://custom.example.com/registry.json""#;
        let config: CommunityConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            config.registry_url,
            "https://custom.example.com/registry.json"
        );
    }

    #[test]
    fn test_registry_source_display() {
        assert_eq!(format!("{}", RegistrySource::Remote), "Remote");
        assert_eq!(format!("{}", RegistrySource::Empty), "Offline");
        assert_eq!(
            format!("{}", RegistrySource::Cached { age_secs: 1800 }),
            "Cached (< 1h ago)"
        );
        assert_eq!(
            format!("{}", RegistrySource::Cached { age_secs: 7200 }),
            "Cached (2h ago)"
        );
        assert_eq!(
            format!("{}", RegistrySource::Cached { age_secs: 172800 }),
            "Cached (2d ago)"
        );
    }
}
