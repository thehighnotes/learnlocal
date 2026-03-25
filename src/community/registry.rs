use std::path::Path;

use crate::community::types::*;

/// Fetch registry from remote URL, falling back to cache on failure.
/// Returns empty registry if no cache and no network.
pub fn fetch_registry(config: &CommunityConfig) -> RegistryResult {
    // Try remote fetch
    match curl_fetch(&config.registry_url, 10) {
        Ok(body) => match serde_json::from_str::<Registry>(&body) {
            Ok(registry) => {
                // Cache for offline use
                if let Err(e) = write_cache(&body) {
                    log::warn!("Failed to cache registry: {}", e);
                }
                RegistryResult {
                    registry,
                    source: RegistrySource::Remote,
                }
            }
            Err(e) => {
                log::warn!("Failed to parse remote registry: {}", e);
                load_cached_or_empty()
            }
        },
        Err(e) => {
            log::debug!("Remote fetch failed: {}", e);
            load_cached_or_empty()
        }
    }
}

/// Load registry from local cache only (offline mode).
pub fn load_cached_registry() -> Option<RegistryResult> {
    let (json, cached_at) = read_cache()?;
    let registry: Registry = serde_json::from_str(&json).ok()?;
    let age_secs = chrono::Utc::now()
        .signed_duration_since(cached_at)
        .num_seconds()
        .max(0) as u64;
    Some(RegistryResult {
        registry,
        source: RegistrySource::Cached { age_secs },
    })
}

/// Search courses by query (matches name, language_display, tags, description).
pub fn search<'a>(courses: &'a [RegistryCourse], query: &str) -> Vec<&'a RegistryCourse> {
    if query.is_empty() {
        return courses.iter().collect();
    }
    let q = query.to_lowercase();
    courses
        .iter()
        .filter(|c| {
            c.name.to_lowercase().contains(&q)
                || c.language_display.to_lowercase().contains(&q)
                || c.language_id.to_lowercase().contains(&q)
                || c.description.to_lowercase().contains(&q)
                || c.id.to_lowercase().contains(&q)
                || c.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .collect()
}

/// Filter courses to only those compatible with the current platform.
#[allow(dead_code)]
pub fn filter_compatible(courses: Vec<&RegistryCourse>) -> Vec<&RegistryCourse> {
    let current_os = std::env::consts::OS;
    courses
        .into_iter()
        .filter(|c| {
            c.platform.as_ref().map_or(true, |p| {
                p == current_os || (p == "linux" && current_os == "linux")
            })
        })
        .collect()
}

/// Check if a course is already installed locally.
pub fn is_installed(course: &RegistryCourse, courses_dir: &Path) -> bool {
    courses_dir.join(&course.id).exists()
}

/// Check if a course's minimum version requirement is met by this binary.
pub fn is_version_compatible(course: &RegistryCourse) -> bool {
    let Some(ref min_ver) = course.min_learnlocal_version else {
        return true;
    };
    let Ok(required) = semver::Version::parse(min_ver) else {
        return true; // Can't parse → assume compatible
    };
    let Ok(current) = semver::Version::parse(env!("CARGO_PKG_VERSION")) else {
        return true;
    };
    current >= required
}

// --- Internal helpers ---

fn load_cached_or_empty() -> RegistryResult {
    match load_cached_registry() {
        Some(result) => result,
        None => RegistryResult {
            registry: Registry {
                version: 1,
                updated_at: String::new(),
                courses: Vec::new(),
            },
            source: RegistrySource::Empty,
        },
    }
}

/// Fetch a URL using curl. Returns the response body as a string.
fn curl_fetch(url: &str, timeout_secs: u64) -> Result<String, String> {
    let output = std::process::Command::new("curl")
        .args(["-fsSL", "--connect-timeout", &timeout_secs.to_string()])
        .arg(url)
        .output()
        .map_err(|e| format!("Failed to run curl: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "curl failed (exit {}): {}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        ));
    }

    String::from_utf8(output.stdout).map_err(|e| format!("Invalid UTF-8 in response: {}", e))
}

/// Cache path: ~/.local/share/learnlocal/registry-cache.json
fn cache_path() -> Option<std::path::PathBuf> {
    dirs::data_dir().map(|d| d.join("learnlocal").join("registry-cache.json"))
}

/// Write registry JSON to cache with a timestamp sidecar.
fn write_cache(json: &str) -> anyhow::Result<()> {
    let path = cache_path().ok_or_else(|| anyhow::anyhow!("No data directory"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write JSON
    std::fs::write(&path, json)?;

    // Write timestamp sidecar
    let ts_path = path.with_extension("timestamp");
    let now = chrono::Utc::now().to_rfc3339();
    std::fs::write(&ts_path, now)?;

    Ok(())
}

/// Read cache file + timestamp. Returns None if missing or corrupt.
fn read_cache() -> Option<(String, chrono::DateTime<chrono::Utc>)> {
    let path = cache_path()?;
    let json = std::fs::read_to_string(&path).ok()?;

    let ts_path = path.with_extension("timestamp");
    let ts_str = std::fs::read_to_string(&ts_path).ok()?;
    let cached_at = chrono::DateTime::parse_from_rfc3339(ts_str.trim())
        .ok()?
        .with_timezone(&chrono::Utc);

    Some((json, cached_at))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_registry() -> Registry {
        serde_json::from_str(
            r#"{
            "version": 1,
            "updated_at": "2026-03-22T00:00:00Z",
            "courses": [
                {
                    "id": "cpp-fundamentals",
                    "name": "C++ Fundamentals",
                    "version": "2.0.0",
                    "author": "Test Author",
                    "description": "Learn C++ from scratch.",
                    "language_id": "cpp",
                    "language_display": "C++",
                    "lessons": 8,
                    "exercises": 58,
                    "download_url": "https://example.com/cpp.tar.gz",
                    "checksum": "sha256:abc",
                    "tags": ["beginner", "cpp", "fundamentals"]
                },
                {
                    "id": "python-fundamentals",
                    "name": "Python Fundamentals",
                    "version": "1.1.0",
                    "author": "Test Author",
                    "description": "Learn Python from zero.",
                    "language_id": "python3",
                    "language_display": "Python",
                    "lessons": 8,
                    "exercises": 57,
                    "platform": "linux",
                    "download_url": "https://example.com/python.tar.gz",
                    "checksum": "sha256:def",
                    "tags": ["beginner", "python"],
                    "min_learnlocal_version": "0.1.0"
                },
                {
                    "id": "rust-fundamentals",
                    "name": "Rust Fundamentals",
                    "version": "1.0.0",
                    "author": "Rustacean",
                    "description": "Systems programming with Rust.",
                    "language_id": "rust",
                    "language_display": "Rust",
                    "lessons": 8,
                    "exercises": 57,
                    "download_url": "https://example.com/rust.tar.gz",
                    "checksum": "sha256:ghi",
                    "tags": ["intermediate", "rust", "systems"],
                    "min_learnlocal_version": "99.0.0"
                }
            ]
        }"#,
        )
        .unwrap()
    }

    #[test]
    fn test_search_by_name() {
        let reg = sample_registry();
        let results = search(&reg.courses, "python");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "python-fundamentals");
    }

    #[test]
    fn test_search_by_language() {
        let reg = sample_registry();
        let results = search(&reg.courses, "C++");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "cpp-fundamentals");
    }

    #[test]
    fn test_search_by_tag() {
        let reg = sample_registry();
        let results = search(&reg.courses, "beginner");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_search_by_id() {
        let reg = sample_registry();
        let results = search(&reg.courses, "rust-fundamentals");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "rust-fundamentals");
    }

    #[test]
    fn test_search_empty_query() {
        let reg = sample_registry();
        let results = search(&reg.courses, "");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_search_no_match() {
        let reg = sample_registry();
        let results = search(&reg.courses, "haskell");
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_case_insensitive() {
        let reg = sample_registry();
        let results = search(&reg.courses, "PYTHON");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_is_installed() {
        let course = RegistryCourse {
            id: "test-course".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            author: "A".to_string(),
            author_github: None,
            description: "D".to_string(),
            language_id: "py".to_string(),
            language_display: "Python".to_string(),
            license: None,
            lessons: 1,
            exercises: 1,
            has_stages: false,
            platform: None,
            provision: "system".to_string(),
            tags: vec![],
            estimated_hours: None,
            download_url: "https://example.com/t.tar.gz".to_string(),
            checksum: "sha256:000".to_string(),
            published_at: String::new(),
            min_learnlocal_version: None,
            avg_rating: None,
            review_count: None,
            downloads: None,
            owner_github: None,
            forked_from: None,
        };
        let tmp = tempfile::tempdir().unwrap();
        assert!(!is_installed(&course, tmp.path()));

        std::fs::create_dir(tmp.path().join("test-course")).unwrap();
        assert!(is_installed(&course, tmp.path()));
    }

    #[test]
    fn test_is_version_compatible() {
        let mut course = RegistryCourse {
            id: "t".to_string(),
            name: "T".to_string(),
            version: "1.0.0".to_string(),
            author: "A".to_string(),
            author_github: None,
            description: "D".to_string(),
            language_id: "py".to_string(),
            language_display: "Python".to_string(),
            license: None,
            lessons: 1,
            exercises: 1,
            has_stages: false,
            platform: None,
            provision: "system".to_string(),
            tags: vec![],
            estimated_hours: None,
            download_url: "https://example.com/t.tar.gz".to_string(),
            checksum: "sha256:000".to_string(),
            published_at: String::new(),
            min_learnlocal_version: None,
            avg_rating: None,
            review_count: None,
            downloads: None,
            owner_github: None,
            forked_from: None,
        };

        // No version requirement → compatible
        assert!(is_version_compatible(&course));

        // Requirement lower than current → compatible
        course.min_learnlocal_version = Some("0.1.0".to_string());
        assert!(is_version_compatible(&course));

        // Requirement higher than current → incompatible
        course.min_learnlocal_version = Some("99.0.0".to_string());
        assert!(!is_version_compatible(&course));
    }

    #[test]
    fn test_cache_round_trip() {
        let json = r#"{"version":1,"updated_at":"2026-01-01T00:00:00Z","courses":[]}"#;

        // Use a temp dir to avoid polluting real cache
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path().join("registry-cache.json");
        let ts = tmp.path().join("registry-cache.timestamp");

        std::fs::write(&cache, json).unwrap();
        let now = chrono::Utc::now().to_rfc3339();
        std::fs::write(&ts, &now).unwrap();

        let content = std::fs::read_to_string(&cache).unwrap();
        let registry: Registry = serde_json::from_str(&content).unwrap();
        assert_eq!(registry.version, 1);
        assert!(registry.courses.is_empty());
    }
}
