use std::path::PathBuf;

use crate::course::types::{Language, Provision};
use crate::exec::registry;
use crate::exec::toolcheck;

/// How a toolchain was resolved for execution.
#[derive(Debug, Clone)]
pub enum ToolchainResolution {
    /// Use tools from system PATH (default behavior)
    System,
    /// Use an embedded in-process runtime (e.g. SQLite)
    Embedded(String),
    /// Use a portable toolchain cached on disk (bin dir to prepend to PATH)
    Portable(PathBuf),
    /// Tools not available — show install instructions
    NotAvailable(String),
}

/// Resolve how to execute a course's language steps.
pub fn resolve_toolchain(language: &Language) -> ToolchainResolution {
    match language.provision {
        Provision::System => {
            let tools = toolcheck::check_language_tools(language);
            let missing: Vec<_> = tools.iter().filter(|t| !t.found).collect();
            if missing.is_empty() {
                ToolchainResolution::System
            } else {
                let hints: Vec<String> = missing
                    .iter()
                    .map(|t| {
                        if let Some(ref hint) = t.install_hint {
                            format!("{}: {}", t.command, hint)
                        } else {
                            format!("{}: not found (no install suggestion available)", t.command)
                        }
                    })
                    .collect();
                ToolchainResolution::NotAvailable(hints.join("\n"))
            }
        }
        Provision::Embedded => {
            if let Some(ref runtime) = language.runtime {
                ToolchainResolution::Embedded(runtime.clone())
            } else {
                ToolchainResolution::NotAvailable(
                    "Embedded provision requires a runtime (e.g. runtime: sqlite)".to_string(),
                )
            }
        }
        Provision::Auto => {
            // Try system first
            let tools = toolcheck::check_language_tools(language);
            let missing: Vec<_> = tools.iter().filter(|t| !t.found).collect();
            if missing.is_empty() {
                return ToolchainResolution::System;
            }

            // Check for cached portable toolchain
            if let Some(cached) = check_cache(&language.id) {
                return ToolchainResolution::Portable(cached);
            }

            // Check if a portable toolchain is available in the registry
            if let Some(entry) = registry::find_toolchain(&language.id) {
                return ToolchainResolution::NotAvailable(format!(
                    "Tools not found. A portable {} {} (~{}MB) can be downloaded.\n\
                     Run this course and press [Y] when prompted to download automatically.",
                    entry.language_id, entry.version, entry.compressed_size_mb,
                ));
            }

            let hints: Vec<String> = missing
                .iter()
                .map(|t| {
                    if let Some(ref hint) = t.install_hint {
                        format!("{}: {}", t.command, hint)
                    } else {
                        format!("{}: not found", t.command)
                    }
                })
                .collect();
            ToolchainResolution::NotAvailable(format!(
                "Tools not found on system:\n{}",
                hints.join("\n")
            ))
        }
        Provision::Manual => {
            let tools = toolcheck::check_language_tools(language);
            let missing: Vec<_> = tools.iter().filter(|t| !t.found).collect();
            if missing.is_empty() {
                ToolchainResolution::System
            } else {
                let hints: Vec<String> = missing
                    .iter()
                    .map(|t| {
                        if let Some(ref hint) = t.install_hint {
                            format!("{}: {}", t.command, hint)
                        } else {
                            format!("{}: not found", t.command)
                        }
                    })
                    .collect();
                ToolchainResolution::NotAvailable(format!(
                    "This course requires manual installation:\n{}",
                    hints.join("\n")
                ))
            }
        }
    }
}

/// Get the toolchains cache directory.
pub fn toolchains_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("learnlocal").join("toolchains"))
}

/// Check if a portable toolchain is already cached for the given language.
fn check_cache(language_id: &str) -> Option<PathBuf> {
    let entry = registry::find_toolchain(language_id)?;
    let cache_dir = toolchains_dir()?;
    let tc_dir = cache_dir.join(format!("{}-{}", entry.language_id, entry.version));
    let bin_dir = tc_dir.join(entry.bin_subdir);
    if bin_dir.is_dir() {
        Some(bin_dir)
    } else {
        None
    }
}

#[allow(dead_code)]
/// Download and install a portable toolchain. Returns the bin directory path.
///
/// Uses curl for download and tar for extraction (available on all POSIX systems).
/// Verifies SHA256 hash after download.
pub fn download_toolchain(language_id: &str) -> Result<PathBuf, String> {
    let entry = registry::find_toolchain(language_id)
        .ok_or_else(|| format!("No portable toolchain available for '{}'", language_id))?;

    let cache_dir = toolchains_dir()
        .ok_or_else(|| "Cannot determine data directory for toolchain cache".to_string())?;

    let tc_dir = cache_dir.join(format!("{}-{}", entry.language_id, entry.version));
    let bin_dir = tc_dir.join(entry.bin_subdir);

    // Already cached?
    if bin_dir.is_dir() {
        return Ok(bin_dir);
    }

    // Create cache directory
    std::fs::create_dir_all(&tc_dir)
        .map_err(|e| format!("Failed to create cache directory: {}", e))?;

    // Download with curl
    let archive_name = entry.url.rsplit('/').next().unwrap_or("archive.tar.gz");
    let archive_path = tc_dir.join(archive_name);

    let curl_status = std::process::Command::new("curl")
        .args(["-fSL", "--progress-bar", "-o"])
        .arg(&archive_path)
        .arg(entry.url)
        .status()
        .map_err(|e| format!("Failed to run curl: {}", e))?;

    if !curl_status.success() {
        let _ = std::fs::remove_dir_all(&tc_dir);
        return Err(format!("Download failed (curl exit {})", curl_status.code().unwrap_or(-1)));
    }

    // Verify SHA256
    let actual_hash = sha256_file(&archive_path)
        .map_err(|e| format!("Failed to compute SHA256: {}", e))?;

    if actual_hash != entry.sha256 {
        let _ = std::fs::remove_dir_all(&tc_dir);
        return Err(format!(
            "SHA256 mismatch!\n  expected: {}\n  got:      {}",
            entry.sha256, actual_hash
        ));
    }

    // Extract
    let tar_flag = match entry.archive_format {
        registry::ArchiveFormat::TarGz => "xzf",
        registry::ArchiveFormat::TarXz => "xJf",
    };

    let tar_status = std::process::Command::new("tar")
        .arg(tar_flag)
        .arg(&archive_path)
        .arg("-C")
        .arg(&tc_dir)
        .status()
        .map_err(|e| format!("Failed to run tar: {}", e))?;

    if !tar_status.success() {
        let _ = std::fs::remove_dir_all(&tc_dir);
        return Err(format!("Extraction failed (tar exit {})", tar_status.code().unwrap_or(-1)));
    }

    // Remove archive to save space
    let _ = std::fs::remove_file(&archive_path);

    if bin_dir.is_dir() {
        Ok(bin_dir)
    } else {
        let _ = std::fs::remove_dir_all(&tc_dir);
        Err(format!(
            "Extraction succeeded but expected bin directory not found: {}",
            bin_dir.display()
        ))
    }
}

/// Compute SHA256 hash of a file using sha256sum command.
fn sha256_file(path: &std::path::Path) -> Result<String, String> {
    let output = std::process::Command::new("sha256sum")
        .arg(path)
        .output()
        .map_err(|e| format!("sha256sum not found: {}", e))?;

    if !output.status.success() {
        return Err("sha256sum failed".to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .split_whitespace()
        .next()
        .map(|s| s.to_string())
        .ok_or_else(|| "Failed to parse sha256sum output".to_string())
}

#[allow(dead_code)]
/// Check if a portable download is available for a language.
pub fn portable_available(language_id: &str) -> bool {
    registry::find_toolchain(language_id).is_some()
}

#[allow(dead_code)]
/// Get download info string for display.
pub fn download_info(language_id: &str) -> Option<String> {
    registry::find_toolchain(language_id).map(|e| {
        format!(
            "{} {} (~{}MB download)",
            e.language_id, e.version, e.compressed_size_mb
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::course::types::{ExecutionLimits, ExecutionStep};

    fn make_language(provision: Provision, runtime: Option<&str>) -> Language {
        Language {
            id: "test".to_string(),
            display_name: "Test".to_string(),
            extension: ".test".to_string(),
            steps: vec![],
            limits: ExecutionLimits::default(),
            provision,
            runtime: runtime.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_resolve_system_no_steps() {
        let lang = make_language(Provision::System, None);
        let result = resolve_toolchain(&lang);
        assert!(matches!(result, ToolchainResolution::System));
    }

    #[test]
    fn test_resolve_embedded_sqlite() {
        let lang = make_language(Provision::Embedded, Some("sqlite"));
        let result = resolve_toolchain(&lang);
        match result {
            ToolchainResolution::Embedded(rt) => assert_eq!(rt, "sqlite"),
            other => panic!("Expected Embedded, got {:?}", other),
        }
    }

    #[test]
    fn test_resolve_embedded_no_runtime() {
        let lang = make_language(Provision::Embedded, None);
        let result = resolve_toolchain(&lang);
        assert!(matches!(result, ToolchainResolution::NotAvailable(_)));
    }

    #[test]
    fn test_resolve_system_missing_tool() {
        let lang = Language {
            id: "test".to_string(),
            display_name: "Test".to_string(),
            extension: ".test".to_string(),
            steps: vec![ExecutionStep {
                name: "run".to_string(),
                command: "nonexistent_tool_xyz_99999".to_string(),
                args: vec![],
                check_exit_code: false,
                capture_output: false,
            }],
            limits: ExecutionLimits::default(),
            provision: Provision::System,
            runtime: None,
        };
        let result = resolve_toolchain(&lang);
        assert!(matches!(result, ToolchainResolution::NotAvailable(_)));
    }

    #[test]
    fn test_resolve_auto_missing_tool() {
        let lang = Language {
            id: "test".to_string(),
            display_name: "Test".to_string(),
            extension: ".test".to_string(),
            steps: vec![ExecutionStep {
                name: "run".to_string(),
                command: "nonexistent_tool_xyz_99999".to_string(),
                args: vec![],
                check_exit_code: false,
                capture_output: false,
            }],
            limits: ExecutionLimits::default(),
            provision: Provision::Auto,
            runtime: None,
        };
        let result = resolve_toolchain(&lang);
        assert!(matches!(result, ToolchainResolution::NotAvailable(_)));
    }

    #[test]
    fn test_resolve_manual_missing_tool() {
        let lang = Language {
            id: "test".to_string(),
            display_name: "Test".to_string(),
            extension: ".test".to_string(),
            steps: vec![ExecutionStep {
                name: "run".to_string(),
                command: "nonexistent_tool_xyz_99999".to_string(),
                args: vec![],
                check_exit_code: false,
                capture_output: false,
            }],
            limits: ExecutionLimits::default(),
            provision: Provision::Manual,
            runtime: None,
        };
        let result = resolve_toolchain(&lang);
        match result {
            ToolchainResolution::NotAvailable(msg) => {
                assert!(msg.contains("manual installation"));
            }
            other => panic!("Expected NotAvailable, got {:?}", other),
        }
    }

    #[test]
    fn test_check_cache_nonexistent() {
        // No cached toolchain should exist for a made-up language
        let result = check_cache("nonexistent_lang_xyz");
        assert!(result.is_none());
    }

    #[test]
    fn test_portable_available() {
        // Python should be in the registry on linux/mac
        if std::env::consts::OS == "linux" || std::env::consts::OS == "macos" {
            assert!(portable_available("python"));
            assert!(portable_available("nodejs"));
            assert!(portable_available("go"));
        }
        assert!(!portable_available("fortran"));
    }

    #[test]
    fn test_download_info() {
        if std::env::consts::OS == "linux" || std::env::consts::OS == "macos" {
            let info = download_info("python");
            assert!(info.is_some());
            assert!(info.unwrap().contains("python"));
        }
    }
}
