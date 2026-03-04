use std::path::Path;

/// Substitute placeholders in a command or argument string.
///
/// Placeholders:
/// - `{dir}` — the sandbox temp directory
/// - `{main}` — the primary source file name
/// - `{output}` — derived binary name (stem of main file, no extension)
/// - `{files}` — space-separated list of all exercise files
pub fn substitute(template: &str, dir: &Path, main_file: &str, all_files: &[String]) -> String {
    let output = Path::new(main_file)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "output".to_string());

    let files_str = all_files
        .iter()
        .map(|f| format!("{}/{}", dir.display(), f))
        .collect::<Vec<_>>()
        .join(" ");

    template
        .replace("{dir}", &dir.to_string_lossy())
        .replace("{main}", main_file)
        .replace("{output}", &output)
        .replace("{files}", &files_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_substitute_all_placeholders() {
        let dir = PathBuf::from("/tmp/sandbox123");
        let result = substitute(
            "g++ -o {dir}/{output} {dir}/{main}",
            &dir,
            "main.cpp",
            &["main.cpp".to_string()],
        );
        assert_eq!(
            result,
            "g++ -o /tmp/sandbox123/main /tmp/sandbox123/main.cpp"
        );
    }

    #[test]
    fn test_substitute_files() {
        let dir = PathBuf::from("/tmp/sb");
        let result = substitute(
            "{files}",
            &dir,
            "main.cpp",
            &["main.cpp".to_string(), "utils.cpp".to_string()],
        );
        assert_eq!(result, "/tmp/sb/main.cpp /tmp/sb/utils.cpp");
    }

    #[test]
    fn test_substitute_no_extension() {
        let dir = PathBuf::from("/tmp/sb");
        let result = substitute("{output}", &dir, "Main.java", &[]);
        assert_eq!(result, "Main");
    }
}
