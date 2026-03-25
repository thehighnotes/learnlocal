use std::path::{Path, PathBuf};

pub struct PreflightResult {
    pub checks: Vec<PreflightCheck>,
}

pub struct PreflightCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

pub struct PackageResult {
    pub archive_path: PathBuf,
    pub manifest: serde_json::Value,
    pub checksum: String,
}

/// Run pre-flight checks on a course directory before packaging.
pub fn preflight_check(course_path: &Path) -> anyhow::Result<PreflightResult> {
    let mut checks = Vec::new();

    // Check: course.yaml exists
    let course_yaml = course_path.join("course.yaml");
    checks.push(PreflightCheck {
        name: "Course file".to_string(),
        passed: course_yaml.exists(),
        message: if course_yaml.exists() {
            "course.yaml found".to_string()
        } else {
            "course.yaml not found".to_string()
        },
    });

    if !course_yaml.exists() {
        return Ok(PreflightResult { checks });
    }

    // Check: course loads successfully
    let course = match crate::course::load_course(course_path) {
        Ok(c) => {
            checks.push(PreflightCheck {
                name: "Course loading".to_string(),
                passed: true,
                message: format!("{} v{}", c.name, c.version),
            });
            Some(c)
        }
        Err(e) => {
            checks.push(PreflightCheck {
                name: "Course loading".to_string(),
                passed: false,
                message: e.to_string(),
            });
            None
        }
    };

    // Check: validation passes
    if let Some(ref c) = course {
        let result = crate::course::validate_course(c);
        let failed: Vec<_> = result.checks.iter().filter(|c| !c.passed).collect();
        checks.push(PreflightCheck {
            name: "Validation".to_string(),
            passed: failed.is_empty(),
            message: if failed.is_empty() {
                format!("{} checks passed", result.checks.len())
            } else {
                format!(
                    "{} issues: {}",
                    failed.len(),
                    failed
                        .iter()
                        .map(|c| c.message.as_str())
                        .collect::<Vec<_>>()
                        .join("; ")
                )
            },
        });
    }

    // Check: description present
    if let Some(ref c) = course {
        checks.push(PreflightCheck {
            name: "Description".to_string(),
            passed: !c.description.is_empty(),
            message: if !c.description.is_empty() {
                "Present".to_string()
            } else {
                "Missing (add 'description:' to course.yaml)".to_string()
            },
        });
    }

    // Check: author present
    if let Some(ref c) = course {
        checks.push(PreflightCheck {
            name: "Author".to_string(),
            passed: !c.author.is_empty(),
            message: if !c.author.is_empty() {
                c.author.clone()
            } else {
                "Missing (add 'author:' to course.yaml)".to_string()
            },
        });
    }

    Ok(PreflightResult { checks })
}

/// Create a tar.gz package from a course directory.
pub fn create_package(course_path: &Path, output_dir: &Path) -> anyhow::Result<PackageResult> {
    let course = crate::course::load_course(course_path)?;
    let course_id = course.name.to_lowercase().replace(' ', "-");

    let filename = format!("{}-{}.tar.gz", course_id, course.version);
    let archive_path = output_dir.join(&filename);

    // Build tar.gz
    let file = std::fs::File::create(&archive_path)?;
    let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);

    // Add all files from course directory, with course_id as prefix
    add_dir_to_tar(&mut builder, course_path, &course_id)?;

    let encoder = builder.into_inner()?;
    encoder.finish()?;

    // Compute checksum
    let checksum = crate::community::download::sha256_file(&archive_path)?;

    // Build manifest
    let total_exercises: usize = course
        .loaded_lessons
        .iter()
        .map(|l| l.loaded_exercises.len())
        .sum();

    let has_stages = course
        .loaded_lessons
        .iter()
        .any(|l| l.loaded_exercises.iter().any(|e| !e.stages.is_empty()));

    let provision_str = match course.language.provision {
        crate::course::types::Provision::System => "system",
        crate::course::types::Provision::Auto => "auto",
        crate::course::types::Provision::Embedded => "embedded",
        crate::course::types::Provision::Manual => "manual",
    };

    let manifest = serde_json::json!({
        "package_version": 1,
        "course_id": course_id,
        "name": course.name,
        "version": course.version,
        "author": course.author,
        "description": course.description,
        "language_id": course.language.id,
        "language_display": course.language.display_name,
        "license": course.license,
        "lessons": course.loaded_lessons.len(),
        "exercises": total_exercises,
        "has_stages": has_stages,
        "platform": course.platform,
        "provision": provision_str,
        "tags": [],
        "estimated_hours": course.estimated_minutes_per_lesson
            .map(|m| (m as f64 * course.loaded_lessons.len() as f64) / 60.0),
        "checksum": format!("sha256:{}", checksum),
        "created_at": chrono::Utc::now().to_rfc3339(),
    });

    Ok(PackageResult {
        archive_path,
        manifest,
        checksum,
    })
}

/// Upload a package to the community server.
pub fn upload_package(
    server_url: &str,
    auth_token: &str,
    package: &PackageResult,
    on_progress: impl Fn(&str),
) -> Result<(), String> {
    on_progress("Uploading package...");

    let manifest_str = package.manifest.to_string();

    let status = std::process::Command::new("curl")
        .args([
            "-fsSL",
            "-X",
            "POST",
            "-H",
            &format!("Authorization: Bearer {}", auth_token),
            "-F",
        ])
        .arg(format!("package=@{}", package.archive_path.display()))
        .args(["-F"])
        .arg(format!("manifest={}", manifest_str))
        .arg(format!("{}/publish", server_url))
        .status()
        .map_err(|e| format!("Failed to run curl: {}", e))?;

    if !status.success() {
        return Err(format!(
            "Upload failed (curl exit {})",
            status.code().unwrap_or(-1)
        ));
    }

    on_progress("Upload complete.");
    Ok(())
}

/// Recursively add a directory to a tar archive, excluding build artifacts.
fn add_dir_to_tar<W: std::io::Write>(
    builder: &mut tar::Builder<W>,
    dir: &Path,
    prefix: &str,
) -> anyhow::Result<()> {
    let skip_names = [
        ".git",
        ".learnlocal-studio.json",
        "__pycache__",
        ".DS_Store",
        "target",
        "node_modules",
    ];

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if skip_names.iter().any(|s| *s == name_str.as_ref()) {
            continue;
        }

        let entry_path = entry.path();
        let archive_path = format!("{}/{}", prefix, name_str);

        if entry.file_type()?.is_dir() {
            add_dir_to_tar(builder, &entry_path, &archive_path)?;
        } else {
            builder.append_path_with_name(&entry_path, &archive_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preflight_nonexistent_dir() {
        let result = preflight_check(Path::new("/tmp/nonexistent-learnlocal-test"));
        assert!(result.is_ok());
        let pf = result.unwrap();
        assert!(!pf.checks[0].passed);
    }
}
