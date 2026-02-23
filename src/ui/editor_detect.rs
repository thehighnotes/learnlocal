use crate::config::EditorType;

const GUI_EDITORS: &[&str] = &[
    "code", "code-insiders", "codium", "vscodium",
    "subl", "sublime_text",
    "zed",
    "gedit", "kate", "kwrite", "mousepad", "pluma", "xed",
    "atom",
    "notepadpp",
];

const TERMINAL_EDITORS: &[&str] = &[
    "vim", "nvim", "neovim",
    "nano", "pico",
    "micro",
    "helix", "hx",
    "emacs",
    "vi", "nvi",
    "joe", "jed", "ne",
    "kakoune", "kak",
];

/// Extract the base command name from an editor string (e.g. "code --wait" → "code").
fn base_command(editor: &str) -> &str {
    editor.split_whitespace().next().unwrap_or(editor)
}

/// Check if the editor command contains a blocking flag (--wait or -w),
/// which means a GUI editor will behave like a terminal editor (blocking).
fn has_wait_flag(editor: &str) -> bool {
    editor.split_whitespace().any(|arg| arg == "--wait" || arg == "-w")
}

/// Detect whether an editor is GUI or Terminal based on its command name.
/// Returns Auto if the editor is unknown.
fn detect_editor_type(editor: &str) -> EditorType {
    let base = base_command(editor);

    // Extract just the filename if it's a path
    let name = base.rsplit('/').next().unwrap_or(base);

    if TERMINAL_EDITORS.iter().any(|&t| t == name) {
        return EditorType::Terminal;
    }

    if GUI_EDITORS.iter().any(|&g| g == name) {
        // GUI editor with --wait flag behaves like terminal (blocks)
        if has_wait_flag(editor) {
            return EditorType::Terminal;
        }
        return EditorType::Gui;
    }

    // Unknown editor defaults to Terminal (safer — assumes blocking)
    EditorType::Terminal
}

/// Resolve the effective editor type given the editor command and config preference.
/// - If config says Terminal or Gui, use that (explicit override).
/// - If config says Auto, detect from the editor name.
/// - If no editor name, default to Terminal.
pub fn resolve_editor_type(editor_name: Option<&str>, config_type: &EditorType) -> EditorType {
    match config_type {
        EditorType::Terminal => EditorType::Terminal,
        EditorType::Gui => EditorType::Gui,
        EditorType::Auto => {
            match editor_name {
                Some(name) if !name.is_empty() => detect_editor_type(name),
                _ => EditorType::Terminal,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vim_is_terminal() {
        assert_eq!(
            resolve_editor_type(Some("vim"), &EditorType::Auto),
            EditorType::Terminal
        );
    }

    #[test]
    fn test_nvim_is_terminal() {
        assert_eq!(
            resolve_editor_type(Some("nvim"), &EditorType::Auto),
            EditorType::Terminal
        );
    }

    #[test]
    fn test_code_is_gui() {
        assert_eq!(
            resolve_editor_type(Some("code"), &EditorType::Auto),
            EditorType::Gui
        );
    }

    #[test]
    fn test_code_wait_is_terminal() {
        assert_eq!(
            resolve_editor_type(Some("code --wait"), &EditorType::Auto),
            EditorType::Terminal
        );
    }

    #[test]
    fn test_code_w_is_terminal() {
        assert_eq!(
            resolve_editor_type(Some("code -w"), &EditorType::Auto),
            EditorType::Terminal
        );
    }

    #[test]
    fn test_subl_is_gui() {
        assert_eq!(
            resolve_editor_type(Some("subl"), &EditorType::Auto),
            EditorType::Gui
        );
    }

    #[test]
    fn test_unknown_defaults_to_terminal() {
        assert_eq!(
            resolve_editor_type(Some("my_custom_editor"), &EditorType::Auto),
            EditorType::Terminal
        );
    }

    #[test]
    fn test_none_defaults_to_terminal() {
        assert_eq!(
            resolve_editor_type(None, &EditorType::Auto),
            EditorType::Terminal
        );
    }

    #[test]
    fn test_explicit_override_terminal() {
        // Even if the editor is "code" (GUI), explicit Terminal wins
        assert_eq!(
            resolve_editor_type(Some("code"), &EditorType::Terminal),
            EditorType::Terminal
        );
    }

    #[test]
    fn test_explicit_override_gui() {
        // Even if the editor is "vim" (Terminal), explicit Gui wins
        assert_eq!(
            resolve_editor_type(Some("vim"), &EditorType::Gui),
            EditorType::Gui
        );
    }

    #[test]
    fn test_nano_is_terminal() {
        assert_eq!(
            resolve_editor_type(Some("nano"), &EditorType::Auto),
            EditorType::Terminal
        );
    }

    #[test]
    fn test_emacs_is_terminal() {
        assert_eq!(
            resolve_editor_type(Some("emacs"), &EditorType::Auto),
            EditorType::Terminal
        );
    }

    #[test]
    fn test_zed_is_gui() {
        assert_eq!(
            resolve_editor_type(Some("zed"), &EditorType::Auto),
            EditorType::Gui
        );
    }

    #[test]
    fn test_full_path_editor() {
        assert_eq!(
            resolve_editor_type(Some("/usr/bin/vim"), &EditorType::Auto),
            EditorType::Terminal
        );
    }
}
