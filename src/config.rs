use serde::{Deserialize, Serialize};

pub use crate::community::types::CommunityConfig;

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ThemePreset {
    #[default]
    Default,
    HighContrast,
}

impl std::fmt::Display for ThemePreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemePreset::Default => write!(f, "default"),
            ThemePreset::HighContrast => write!(f, "high-contrast"),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxLevelPref {
    #[default]
    Auto,
    Basic,
    Contained,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum EditorType {
    #[default]
    Auto,
    Terminal,
    Gui,
}

impl std::fmt::Display for EditorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditorType::Auto => write!(f, "auto"),
            EditorType::Terminal => write!(f, "terminal"),
            EditorType::Gui => write!(f, "gui"),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub editor: Option<String>,
    #[serde(default, alias = "sandbox-level")]
    pub sandbox_level: SandboxLevelPref,
    #[serde(default, alias = "editor-type")]
    pub editor_type: EditorType,
    #[serde(default)]
    pub theme: ThemePreset,
    #[cfg(feature = "llm")]
    #[serde(default)]
    pub llm: crate::llm::config::LlmConfig,
    #[cfg(not(feature = "llm"))]
    #[serde(default)]
    pub llm: serde_yaml::Value,
    #[serde(default)]
    pub community: CommunityConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: None,
            sandbox_level: SandboxLevelPref::Auto,
            editor_type: EditorType::Auto,
            theme: ThemePreset::Default,
            #[cfg(feature = "llm")]
            llm: crate::llm::config::LlmConfig::default(),
            #[cfg(not(feature = "llm"))]
            llm: serde_yaml::Value::Null,
            community: CommunityConfig::default(),
        }
    }
}

impl Config {
    /// Load config from ~/.config/learnlocal/config.yaml
    /// Returns Default on missing file or parse error.
    pub fn load() -> Self {
        let path = match dirs::config_dir() {
            Some(dir) => dir.join("learnlocal").join("config.yaml"),
            None => return Config::default(),
        };

        match std::fs::read_to_string(&path) {
            Ok(contents) => match serde_yaml::from_str(&contents) {
                Ok(config) => {
                    log::debug!("Config loaded from {}", path.display());
                    config
                }
                Err(e) => {
                    eprintln!(
                        "{} Failed to parse {}: {} — using defaults",
                        crate::cli_fmt::yellow("Warning:"),
                        path.display(),
                        e
                    );
                    log::warn!("Config parse error: {}", e);
                    Config::default()
                }
            },
            Err(_) => {
                log::debug!("No config file at {}, using defaults", path.display());
                Config::default()
            }
        }
    }

    /// Save config to ~/.config/learnlocal/config.yaml (atomic write).
    pub fn save(&self) -> anyhow::Result<()> {
        let dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine config directory"))?
            .join("learnlocal");
        std::fs::create_dir_all(&dir)?;

        let path = dir.join("config.yaml");
        let yaml = serde_yaml::to_string(self)?;

        let tmp_path = path.with_extension("yaml.tmp");
        std::fs::write(&tmp_path, &yaml)?;
        std::fs::rename(&tmp_path, &path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.editor, None);
        assert_eq!(config.sandbox_level, SandboxLevelPref::Auto);
        assert_eq!(config.editor_type, EditorType::Auto);
    }

    #[test]
    fn test_config_parse_full() {
        let yaml = r#"
editor: "nvim"
sandbox-level: contained
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.editor.as_deref(), Some("nvim"));
        assert_eq!(config.sandbox_level, SandboxLevelPref::Contained);
    }

    #[test]
    fn test_config_parse_partial() {
        let yaml = r#"
editor: "code --wait"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.editor.as_deref(), Some("code --wait"));
        assert_eq!(config.sandbox_level, SandboxLevelPref::Auto);
    }

    #[test]
    fn test_config_unknown_keys_ignored() {
        let yaml = r#"
editor: "vim"
some_future_field: true
llm:
  backend: ollama
  ollama:
    model: gpt-4
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.editor.as_deref(), Some("vim"));
    }

    #[test]
    fn test_config_empty_yaml() {
        let yaml = "{}";
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.editor, None);
        assert_eq!(config.sandbox_level, SandboxLevelPref::Auto);
    }

    #[test]
    fn test_config_editor_priority_over_env() {
        let config = Config {
            editor: Some("custom-editor".to_string()),
            ..Config::default()
        };
        assert_eq!(config.editor.as_deref(), Some("custom-editor"));
    }

    #[test]
    fn test_config_editor_type_parse() {
        let yaml = r#"
editor: "vim"
editor-type: gui
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.editor_type, EditorType::Gui);
    }

    #[test]
    fn test_config_editor_type_default_when_missing() {
        let yaml = r#"
editor: "vim"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.editor_type, EditorType::Auto);
    }

    #[test]
    fn test_config_theme_high_contrast() {
        let yaml = r#"
theme: high-contrast
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.theme, ThemePreset::HighContrast);
    }

    #[test]
    fn test_config_theme_default_when_missing() {
        let yaml = r#"
editor: "vim"
"#;
        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.theme, ThemePreset::Default);
    }
}
