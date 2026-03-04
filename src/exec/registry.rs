/// Registry of portable toolchain downloads compiled into the binary.
///
/// Each entry describes a downloadable toolchain for a specific language + platform.
/// The `provision: auto` mode checks this registry when system tools are missing.

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ToolchainEntry {
    pub language_id: &'static str,
    pub version: &'static str,
    pub platform: &'static str,
    pub arch: &'static str,
    pub url: &'static str,
    pub sha256: &'static str,
    pub compressed_size_mb: u32,
    pub archive_format: ArchiveFormat,
    pub bin_subdir: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub enum ArchiveFormat {
    TarGz,
    TarXz,
}

/// Find a registry entry matching the given language and current platform.
pub fn find_toolchain(language_id: &str) -> Option<&'static ToolchainEntry> {
    let platform = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    // Map Rust arch names to registry names
    let reg_arch = match arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => return None,
    };
    let reg_platform = match platform {
        "linux" => "linux",
        "macos" => "darwin",
        _ => return None,
    };

    REGISTRY
        .iter()
        .find(|e| e.language_id == language_id && e.platform == reg_platform && e.arch == reg_arch)
}

/// Static registry of portable toolchains.
///
/// URLs point to official distribution tarballs. SHA256 hashes ensure integrity.
/// Entries are added as courses adopt `provision: auto`.
static REGISTRY: &[ToolchainEntry] = &[
    // ─── Python 3.13 (python-build-standalone) ───────────────────────
    ToolchainEntry {
        language_id: "python",
        version: "3.13.1",
        platform: "linux",
        arch: "x86_64",
        url: "https://github.com/indygreg/python-build-standalone/releases/download/20241206/cpython-3.13.1+20241206-x86_64-unknown-linux-gnu-install_only_stripped.tar.gz",
        sha256: "0e02cba42afbed0bf1e21d2d85bfa5292e4e2e0bbbfc477e3a77401c0340ead0",
        compressed_size_mb: 30,
        archive_format: ArchiveFormat::TarGz,
        bin_subdir: "python/bin",
    },
    ToolchainEntry {
        language_id: "python",
        version: "3.13.1",
        platform: "linux",
        arch: "aarch64",
        url: "https://github.com/indygreg/python-build-standalone/releases/download/20241206/cpython-3.13.1+20241206-aarch64-unknown-linux-gnu-install_only_stripped.tar.gz",
        sha256: "eb7e5658e73ad17e3e63b1ffeb65f6527d1d0ed66d43d1a95ba42f2fdc202e2b",
        compressed_size_mb: 30,
        archive_format: ArchiveFormat::TarGz,
        bin_subdir: "python/bin",
    },
    ToolchainEntry {
        language_id: "python",
        version: "3.13.1",
        platform: "darwin",
        arch: "x86_64",
        url: "https://github.com/indygreg/python-build-standalone/releases/download/20241206/cpython-3.13.1+20241206-x86_64-apple-darwin-install_only_stripped.tar.gz",
        sha256: "e2bedfe0a35d09040fa9c07e0fc57ebadbb5a2f3aef2c7e43a9d87207ccae3b1",
        compressed_size_mb: 30,
        archive_format: ArchiveFormat::TarGz,
        bin_subdir: "python/bin",
    },
    ToolchainEntry {
        language_id: "python",
        version: "3.13.1",
        platform: "darwin",
        arch: "aarch64",
        url: "https://github.com/indygreg/python-build-standalone/releases/download/20241206/cpython-3.13.1+20241206-aarch64-apple-darwin-install_only_stripped.tar.gz",
        sha256: "c6d52a8c8e06f36bc3116f0f9ca39e6e13d5a8d6a0b590ed6f453f39ac9de5da",
        compressed_size_mb: 30,
        archive_format: ArchiveFormat::TarGz,
        bin_subdir: "python/bin",
    },
    // ─── Node.js 22 (official) ───────────────────────────────────────
    ToolchainEntry {
        language_id: "nodejs",
        version: "22.12.0",
        platform: "linux",
        arch: "x86_64",
        url: "https://nodejs.org/dist/v22.12.0/node-v22.12.0-linux-x64.tar.xz",
        sha256: "52d4bbb7668854a5067b8def1bdc517eb5be3e73840bd6727f0555b8b77fc3da",
        compressed_size_mb: 25,
        archive_format: ArchiveFormat::TarXz,
        bin_subdir: "node-v22.12.0-linux-x64/bin",
    },
    ToolchainEntry {
        language_id: "nodejs",
        version: "22.12.0",
        platform: "linux",
        arch: "aarch64",
        url: "https://nodejs.org/dist/v22.12.0/node-v22.12.0-linux-arm64.tar.xz",
        sha256: "e41c89feeebe310693de6b2ee74ef1aad934b771c54d4cbff37775dcb89cb5d3",
        compressed_size_mb: 25,
        archive_format: ArchiveFormat::TarXz,
        bin_subdir: "node-v22.12.0-linux-arm64/bin",
    },
    ToolchainEntry {
        language_id: "nodejs",
        version: "22.12.0",
        platform: "darwin",
        arch: "x86_64",
        url: "https://nodejs.org/dist/v22.12.0/node-v22.12.0-darwin-x64.tar.gz",
        sha256: "7f3a2f95f80c43db3dafdfc0ef3a5f17bc54a43c69ec2389a0d41a63e1dd6e3e",
        compressed_size_mb: 25,
        archive_format: ArchiveFormat::TarGz,
        bin_subdir: "node-v22.12.0-darwin-x64/bin",
    },
    ToolchainEntry {
        language_id: "nodejs",
        version: "22.12.0",
        platform: "darwin",
        arch: "aarch64",
        url: "https://nodejs.org/dist/v22.12.0/node-v22.12.0-darwin-arm64.tar.gz",
        sha256: "dd9ed4e48b65a9ec94e2626e0e19523da5c807b8d8e63eb5a0ea1fcab23a6a04",
        compressed_size_mb: 25,
        archive_format: ArchiveFormat::TarGz,
        bin_subdir: "node-v22.12.0-darwin-arm64/bin",
    },
    // ─── Go 1.23 (official) ─────────────────────────────────────────
    ToolchainEntry {
        language_id: "go",
        version: "1.23.4",
        platform: "linux",
        arch: "x86_64",
        url: "https://go.dev/dl/go1.23.4.linux-amd64.tar.gz",
        sha256: "6924efde5de86fe277676e929dc9917d466571f1a507de904c8ebbc6f56ce2e0",
        compressed_size_mb: 64,
        archive_format: ArchiveFormat::TarGz,
        bin_subdir: "go/bin",
    },
    ToolchainEntry {
        language_id: "go",
        version: "1.23.4",
        platform: "linux",
        arch: "aarch64",
        url: "https://go.dev/dl/go1.23.4.linux-arm64.tar.gz",
        sha256: "16e5017863a7f6071363782b1b8042eb12c6ca4f4cd71528b2123f0a1275b13e",
        compressed_size_mb: 64,
        archive_format: ArchiveFormat::TarGz,
        bin_subdir: "go/bin",
    },
    ToolchainEntry {
        language_id: "go",
        version: "1.23.4",
        platform: "darwin",
        arch: "x86_64",
        url: "https://go.dev/dl/go1.23.4.darwin-amd64.tar.gz",
        sha256: "6700067389a53ca1a0d19199a3024ebc730c9b95db56bc01e68a20a58a5588b3",
        compressed_size_mb: 64,
        archive_format: ArchiveFormat::TarGz,
        bin_subdir: "go/bin",
    },
    ToolchainEntry {
        language_id: "go",
        version: "1.23.4",
        platform: "darwin",
        arch: "aarch64",
        url: "https://go.dev/dl/go1.23.4.darwin-arm64.tar.gz",
        sha256: "87d2bb0ad4fe24d2a0685a55df321e0efe4296419a9b27f6ab99e1ed2f8a9bb2",
        compressed_size_mb: 64,
        archive_format: ArchiveFormat::TarGz,
        bin_subdir: "go/bin",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_has_entries() {
        assert!(!REGISTRY.is_empty());
    }

    #[test]
    fn test_find_toolchain_python() {
        // On any linux/mac system this should find an entry
        let result = find_toolchain("python");
        // May or may not find depending on OS/arch, but shouldn't panic
        if std::env::consts::OS == "linux" || std::env::consts::OS == "macos" {
            if std::env::consts::ARCH == "x86_64" || std::env::consts::ARCH == "aarch64" {
                assert!(result.is_some());
                let entry = result.unwrap();
                assert_eq!(entry.language_id, "python");
            }
        }
    }

    #[test]
    fn test_find_toolchain_unknown() {
        let result = find_toolchain("fortran");
        assert!(result.is_none());
    }

    #[test]
    fn test_all_entries_have_sha256() {
        for entry in REGISTRY {
            assert!(
                !entry.sha256.is_empty(),
                "entry {} {} missing sha256",
                entry.language_id,
                entry.platform
            );
            assert_eq!(
                entry.sha256.len(),
                64,
                "sha256 wrong length for {} {}",
                entry.language_id,
                entry.platform
            );
        }
    }

    #[test]
    fn test_all_entries_have_valid_format() {
        for entry in REGISTRY {
            match entry.archive_format {
                ArchiveFormat::TarGz => {
                    assert!(entry.url.ends_with(".tar.gz") || entry.url.ends_with(".tgz"))
                }
                ArchiveFormat::TarXz => assert!(entry.url.ends_with(".tar.xz")),
            }
        }
    }
}
