use std::path::{Path, PathBuf};

use crate::community::types::RegistryCourse;

/// Stages of the download/install process, reported via callback.
#[derive(Debug, Clone)]
pub enum DownloadProgress {
    Downloading,
    Verifying,
    Extracting,
    Validating,
    Installing,
    Complete,
}

/// Outcome of an install attempt.
#[derive(Debug)]
pub enum DownloadResult {
    /// Successfully installed.
    Success {
        course_id: String,
        install_path: PathBuf,
    },
    /// Course directory already exists.
    AlreadyInstalled {
        course_id: String,
        install_path: PathBuf,
    },
    /// SHA-256 checksum did not match.
    ChecksumMismatch { expected: String, actual: String },
    /// Network or curl error.
    NetworkError(String),
    /// tar/flate2 extraction error.
    ExtractionError(String),
    /// Course failed structural validation after extraction.
    ValidationFailed(String),
    /// Binary version too old for this course.
    IncompatibleVersion { required: String, current: String },
    /// Course requires a different OS.
    PlatformMismatch { required: String, current: String },
}

impl DownloadResult {
    #[allow(dead_code)]
    pub fn is_success(&self) -> bool {
        matches!(self, DownloadResult::Success { .. })
    }
}

/// Download and install a course from the community registry.
///
/// Flow: pre-flight → download → verify checksum → extract → validate → move to courses_dir.
pub fn install_course(
    course: &RegistryCourse,
    courses_dir: &Path,
    on_progress: impl Fn(DownloadProgress),
) -> DownloadResult {
    // Pre-flight: platform check
    if let Some(ref required_platform) = course.platform {
        let current = std::env::consts::OS;
        if required_platform != current {
            return DownloadResult::PlatformMismatch {
                required: required_platform.clone(),
                current: current.to_string(),
            };
        }
    }

    // Pre-flight: version check
    if let Some(ref min_ver) = course.min_learnlocal_version {
        if let (Ok(required), Ok(current)) = (
            semver::Version::parse(min_ver),
            semver::Version::parse(env!("CARGO_PKG_VERSION")),
        ) {
            if current < required {
                return DownloadResult::IncompatibleVersion {
                    required: min_ver.clone(),
                    current: env!("CARGO_PKG_VERSION").to_string(),
                };
            }
        }
    }

    // Pre-flight: already installed?
    let install_path = courses_dir.join(&course.id);
    if install_path.exists() {
        return DownloadResult::AlreadyInstalled {
            course_id: course.id.clone(),
            install_path,
        };
    }

    // Create temp dir for download + extraction
    let tmp_dir = match tempfile::tempdir() {
        Ok(d) => d,
        Err(e) => {
            return DownloadResult::ExtractionError(format!("Failed to create temp dir: {}", e))
        }
    };

    // Download
    on_progress(DownloadProgress::Downloading);
    let archive_path = tmp_dir.path().join("course.tar.gz");
    if let Err(e) = download_to_file(&course.download_url, &archive_path) {
        return DownloadResult::NetworkError(e);
    }

    // Verify checksum
    on_progress(DownloadProgress::Verifying);
    let (algo, expected_hex) = match parse_checksum(&course.checksum) {
        Ok(pair) => pair,
        Err(e) => {
            return DownloadResult::ChecksumMismatch {
                expected: course.checksum.clone(),
                actual: format!("(parse error: {})", e),
            }
        }
    };

    if algo != "sha256" {
        return DownloadResult::ChecksumMismatch {
            expected: course.checksum.clone(),
            actual: format!("(unsupported algorithm: {})", algo),
        };
    }

    let actual_hex = match sha256_file(&archive_path) {
        Ok(h) => h,
        Err(e) => {
            return DownloadResult::ChecksumMismatch {
                expected: expected_hex.to_string(),
                actual: format!("(hash error: {})", e),
            }
        }
    };

    if actual_hex != expected_hex {
        return DownloadResult::ChecksumMismatch {
            expected: expected_hex.to_string(),
            actual: actual_hex,
        };
    }

    // Extract
    on_progress(DownloadProgress::Extracting);
    let extract_dir = tmp_dir.path().join("extracted");
    if let Err(e) = std::fs::create_dir(&extract_dir) {
        return DownloadResult::ExtractionError(format!("Failed to create extraction dir: {}", e));
    }
    if let Err(e) = extract_tar_gz(&archive_path, &extract_dir) {
        return DownloadResult::ExtractionError(e);
    }

    // Find the course root (may be nested in a single top-level directory)
    let course_root = match find_course_root(&extract_dir) {
        Some(p) => p,
        None => {
            return DownloadResult::ValidationFailed(
                "No course.yaml found in extracted archive".to_string(),
            );
        }
    };

    // Validate
    on_progress(DownloadProgress::Validating);
    match crate::course::load_course(&course_root) {
        Ok(loaded) => {
            let result = crate::course::validate_course(&loaded);
            if !result.all_passed() {
                let failures: Vec<_> = result
                    .checks
                    .iter()
                    .filter(|c| !c.passed)
                    .map(|c| c.message.clone())
                    .collect();
                return DownloadResult::ValidationFailed(failures.join("; "));
            }
        }
        Err(e) => {
            return DownloadResult::ValidationFailed(format!("Failed to load course: {}", e));
        }
    }

    // Install: move to courses_dir
    on_progress(DownloadProgress::Installing);
    if let Some(parent) = install_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            return DownloadResult::ExtractionError(format!("Failed to create courses dir: {}", e));
        }
    }

    // Try rename first (same filesystem), fall back to recursive copy
    if std::fs::rename(&course_root, &install_path).is_err() {
        if let Err(e) = copy_dir_recursive(&course_root, &install_path) {
            return DownloadResult::ExtractionError(format!("Failed to install course: {}", e));
        }
    }

    on_progress(DownloadProgress::Complete);
    DownloadResult::Success {
        course_id: course.id.clone(),
        install_path,
    }
}

/// Download a file using curl.
fn download_to_file(url: &str, dest: &Path) -> Result<(), String> {
    let status = std::process::Command::new("curl")
        .args(["-fSL", "--connect-timeout", "30", "-o"])
        .arg(dest)
        .arg(url)
        .status()
        .map_err(|e| format!("Failed to run curl: {}", e))?;

    if !status.success() {
        return Err(format!(
            "Download failed (curl exit {})",
            status.code().unwrap_or(-1)
        ));
    }
    Ok(())
}

/// Compute SHA-256 hex digest of a file.
pub fn sha256_file(path: &Path) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

/// Parse a "algorithm:hexdigest" checksum string.
pub fn parse_checksum(checksum: &str) -> anyhow::Result<(&str, &str)> {
    let parts: Vec<&str> = checksum.splitn(2, ':').collect();
    if parts.len() != 2 {
        anyhow::bail!(
            "Invalid checksum format '{}' — expected 'algorithm:hexdigest'",
            checksum
        );
    }
    Ok((parts[0], parts[1]))
}

/// Extract a tar.gz archive to a directory with path traversal protection.
pub fn extract_tar_gz(archive_path: &Path, dest: &Path) -> Result<(), String> {
    use flate2::read::GzDecoder;

    // First pass: scan for path traversal
    let file =
        std::fs::File::open(archive_path).map_err(|e| format!("Cannot open archive: {}", e))?;
    let decoder = GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    for entry in archive
        .entries()
        .map_err(|e| format!("Cannot read archive entries: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Bad archive entry: {}", e))?;
        let path = entry
            .path()
            .map_err(|e| format!("Bad path in entry: {}", e))?;

        if path.is_absolute() {
            return Err(format!(
                "Archive contains absolute path: {}",
                path.display()
            ));
        }
        if path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err(format!(
                "Archive contains path traversal: {}",
                path.display()
            ));
        }
    }

    // Second pass: extract
    let file =
        std::fs::File::open(archive_path).map_err(|e| format!("Cannot reopen archive: {}", e))?;
    let decoder = GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(dest)
        .map_err(|e| format!("Extraction failed: {}", e))?;

    Ok(())
}

/// Find the course root directory (the one containing course.yaml) in extracted files.
/// Handles both flat extraction and single-directory wrapper.
fn find_course_root(extract_dir: &Path) -> Option<PathBuf> {
    // Check if course.yaml is directly in extract_dir
    if extract_dir.join("course.yaml").exists() {
        return Some(extract_dir.to_path_buf());
    }

    // Check one level deep (single wrapping directory)
    if let Ok(entries) = std::fs::read_dir(extract_dir) {
        let dirs: Vec<_> = entries
            .flatten()
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect();

        if dirs.len() == 1 {
            let candidate = dirs[0].path();
            if candidate.join("course.yaml").exists() {
                return Some(candidate);
            }
        }
    }

    None
}

/// Recursively copy a directory tree.
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let dest_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), &dest_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_checksum_valid() {
        let (algo, hex) = parse_checksum("sha256:abcdef1234567890").unwrap();
        assert_eq!(algo, "sha256");
        assert_eq!(hex, "abcdef1234567890");
    }

    #[test]
    fn test_parse_checksum_invalid() {
        assert!(parse_checksum("nocolon").is_err());
        assert!(parse_checksum("").is_err());
    }

    #[test]
    fn test_parse_checksum_with_colons_in_digest() {
        let (algo, hex) = parse_checksum("sha256:abc:def").unwrap();
        assert_eq!(algo, "sha256");
        assert_eq!(hex, "abc:def");
    }

    #[test]
    fn test_sha256_file() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), b"hello world\n").unwrap();
        let hash = sha256_file(tmp.path()).unwrap();
        assert_eq!(
            hash,
            "a948904f2f0f479b8f8197694b30184b0d2ed1c1cd2a1ec0fb85d299a192a447"
        );
    }

    #[test]
    fn test_sha256_empty_file() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), b"").unwrap();
        let hash = sha256_file(tmp.path()).unwrap();
        // sha256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_extract_tar_gz_round_trip() {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let tmp = tempfile::tempdir().unwrap();
        let archive_path = tmp.path().join("test.tar.gz");

        // Create a tar.gz with a single file
        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let encoder = GzEncoder::new(file, Compression::default());
            let mut builder = tar::Builder::new(encoder);

            let content = b"course data here";
            let mut header = tar::Header::new_gnu();
            header.set_size(content.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder
                .append_data(&mut header, "my-course/course.yaml", &content[..])
                .unwrap();
            builder.finish().unwrap();
        }

        // Extract
        let extract_dir = tmp.path().join("out");
        std::fs::create_dir(&extract_dir).unwrap();
        extract_tar_gz(&archive_path, &extract_dir).unwrap();

        // Verify
        let extracted = extract_dir.join("my-course").join("course.yaml");
        assert!(extracted.exists());
        let content = std::fs::read_to_string(extracted).unwrap();
        assert_eq!(content, "course data here");
    }

    #[test]
    fn test_extract_rejects_path_traversal() {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let tmp = tempfile::tempdir().unwrap();
        let archive_path = tmp.path().join("evil.tar.gz");

        // The tar crate builder rejects malicious paths, so we craft raw tar bytes
        // to test our defense-in-depth scanning layer.
        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let encoder = GzEncoder::new(file, Compression::default());
            let raw_tar = build_raw_tar_entry(b"../../../etc/evil.txt", b"malicious");
            let mut encoder = encoder;
            std::io::Write::write_all(&mut encoder, &raw_tar).ok();
            encoder.finish().unwrap();
        }

        let extract_dir = tmp.path().join("out");
        std::fs::create_dir(&extract_dir).unwrap();
        let result = extract_tar_gz(&archive_path, &extract_dir);
        // Should fail — either our scan or tar's built-in protection catches it
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_rejects_absolute_path() {
        use flate2::write::GzEncoder;
        use flate2::Compression;

        let tmp = tempfile::tempdir().unwrap();
        let archive_path = tmp.path().join("abs.tar.gz");

        {
            let file = std::fs::File::create(&archive_path).unwrap();
            let encoder = GzEncoder::new(file, Compression::default());
            let raw_tar = build_raw_tar_entry(b"/tmp/evil.txt", b"bad");
            let mut encoder = encoder;
            std::io::Write::write_all(&mut encoder, &raw_tar).ok();
            encoder.finish().unwrap();
        }

        let extract_dir = tmp.path().join("out");
        std::fs::create_dir(&extract_dir).unwrap();
        let result = extract_tar_gz(&archive_path, &extract_dir);
        assert!(result.is_err());
    }

    /// Build a raw POSIX tar entry with a given name and content, bypassing builder validation.
    fn build_raw_tar_entry(name: &[u8], content: &[u8]) -> Vec<u8> {
        let mut header = [0u8; 512];
        // Name field: bytes 0..100
        let name_len = name.len().min(100);
        header[..name_len].copy_from_slice(&name[..name_len]);
        // Mode: bytes 100..108
        header[100..107].copy_from_slice(b"0000644");
        // UID: bytes 108..116
        header[108..115].copy_from_slice(b"0001000");
        // GID: bytes 116..124
        header[116..123].copy_from_slice(b"0001000");
        // Size: bytes 124..136 (octal)
        let size_str = format!("{:011o}", content.len());
        header[124..135].copy_from_slice(size_str.as_bytes());
        // Mtime: bytes 136..148
        header[136..147].copy_from_slice(b"14542671045");
        // Type flag: '0' = regular file
        header[156] = b'0';
        // Magic: bytes 257..263 = "ustar\0"
        header[257..263].copy_from_slice(b"ustar\0");
        // Version: bytes 263..265 = "00"
        header[263..265].copy_from_slice(b"00");
        // Compute checksum: bytes 148..156 (8 bytes, initially spaces)
        header[148..156].copy_from_slice(b"        ");
        let cksum: u32 = header.iter().map(|&b| b as u32).sum();
        let cksum_str = format!("{:06o}\0 ", cksum);
        header[148..156].copy_from_slice(cksum_str.as_bytes());

        let mut result = header.to_vec();
        result.extend_from_slice(content);
        // Pad to 512-byte boundary
        let padding = (512 - (content.len() % 512)) % 512;
        result.extend(std::iter::repeat(0u8).take(padding));
        // Two 512-byte zero blocks to end the archive
        result.extend(std::iter::repeat(0u8).take(1024));
        result
    }

    #[test]
    fn test_find_course_root_flat() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("course.yaml"), "name: test").unwrap();
        assert_eq!(find_course_root(tmp.path()), Some(tmp.path().to_path_buf()));
    }

    #[test]
    fn test_find_course_root_nested() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("my-course");
        std::fs::create_dir(&nested).unwrap();
        std::fs::write(nested.join("course.yaml"), "name: test").unwrap();
        assert_eq!(find_course_root(tmp.path()), Some(nested));
    }

    #[test]
    fn test_find_course_root_none() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(find_course_root(tmp.path()), None);
    }

    #[test]
    fn test_download_result_is_success() {
        let success = DownloadResult::Success {
            course_id: "test".to_string(),
            install_path: PathBuf::from("/tmp/test"),
        };
        assert!(success.is_success());

        let fail = DownloadResult::NetworkError("timeout".to_string());
        assert!(!fail.is_success());
    }

    #[test]
    fn test_copy_dir_recursive() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        std::fs::create_dir_all(src.join("sub")).unwrap();
        std::fs::write(src.join("a.txt"), "hello").unwrap();
        std::fs::write(src.join("sub").join("b.txt"), "world").unwrap();

        copy_dir_recursive(&src, &dst).unwrap();

        assert_eq!(std::fs::read_to_string(dst.join("a.txt")).unwrap(), "hello");
        assert_eq!(
            std::fs::read_to_string(dst.join("sub").join("b.txt")).unwrap(),
            "world"
        );
    }
}
