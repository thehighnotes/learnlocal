use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::Command;

use crate::error::{LearnLocalError, Result};

/// Detect the user's preferred editor.
/// Priority: config → $LEARNLOCAL_EDITOR → $VISUAL → $EDITOR → vi
pub fn detect_editor(config_editor: Option<&str>) -> Option<String> {
    config_editor
        .map(|s| s.to_string())
        .or_else(|| std::env::var("LEARNLOCAL_EDITOR").ok())
        .or_else(|| std::env::var("VISUAL").ok())
        .or_else(|| std::env::var("EDITOR").ok())
        .or_else(|| {
            // Check if vi is available
            if Command::new("which")
                .arg("vi")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                Some("vi".to_string())
            } else {
                None
            }
        })
}

/// Open a file in the user's editor. Blocks until the editor exits.
/// Returns the (possibly modified) file contents.
/// Convenience wrapper for edit_file_with_config with no config editor.
#[allow(dead_code)]
pub fn edit_file(file_path: &Path) -> Result<String> {
    edit_file_with_config(file_path, None)
}

/// Open a file in the user's editor, with optional config-specified editor.
/// Blocks until the editor exits. Returns the (possibly modified) file contents.
pub fn edit_file_with_config(file_path: &Path, config_editor: Option<&str>) -> Result<String> {
    let editor = detect_editor(config_editor);

    match editor {
        Some(editor) => {
            let status = Command::new(&editor).arg(file_path).status().map_err(|e| {
                LearnLocalError::Editor(format!("Failed to launch editor '{}': {}", editor, e))
            })?;

            if !status.success() {
                return Err(LearnLocalError::Editor(format!(
                    "Editor '{}' exited with non-zero status",
                    editor
                )));
            }

            std::fs::read_to_string(file_path).map_err(|e| {
                LearnLocalError::Editor(format!("Failed to read back file after editing: {}", e))
            })
        }
        None => {
            // Minimal line-based fallback
            minimal_line_editor(file_path)
        }
    }
}

/// Minimal line-based editor for when $EDITOR is not set.
fn minimal_line_editor(file_path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(file_path)?;
    let lines: Vec<&str> = content.lines().collect();

    println!("\n--- Minimal Editor (set $EDITOR for a better experience) ---");
    println!("Current file contents:\n");

    for (i, line) in lines.iter().enumerate() {
        println!("  {:3}  {}", i + 1, line);
    }

    println!("\nEnter line number to edit (or 'done' to finish):");

    let stdin = io::stdin();
    let mut new_lines: Vec<String> = lines.iter().map(|l| l.to_string()).collect();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;
        let input = input.trim();

        if input == "done" || input.is_empty() {
            break;
        }

        if let Ok(line_num) = input.parse::<usize>() {
            if line_num >= 1 && line_num <= new_lines.len() {
                println!(
                    "Line {} (current: \"{}\")",
                    line_num,
                    new_lines[line_num - 1]
                );
                print!("> ");
                io::stdout().flush()?;

                let mut new_content = String::new();
                stdin.lock().read_line(&mut new_content)?;
                new_lines[line_num - 1] = new_content.trim_end_matches('\n').to_string();
            } else {
                println!("Line number out of range (1-{})", new_lines.len());
            }
        } else {
            println!("Enter a line number or 'done'");
        }
    }

    let result = new_lines.join("\n") + "\n";
    std::fs::write(file_path, &result)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_editor_env() {
        // This test depends on environment, so just verify it returns something or None
        let editor = detect_editor(None);
        // On most systems, at least vi should be available
        // But we can't guarantee it in all CI environments
        let _ = editor;
    }

    #[test]
    fn test_detect_editor_config_priority() {
        let editor = detect_editor(Some("custom-editor"));
        assert_eq!(editor, Some("custom-editor".to_string()));
    }
}
