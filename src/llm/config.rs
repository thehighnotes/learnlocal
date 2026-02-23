use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_backend")]
    pub backend: String,
    #[serde(default)]
    pub ollama: OllamaConfig,
    #[serde(default)]
    pub settings: LlmSettings,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            backend: default_backend(),
            ollama: OllamaConfig::default(),
            settings: LlmSettings::default(),
        }
    }
}

fn default_backend() -> String {
    "ollama".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OllamaConfig {
    #[serde(default = "default_url")]
    pub url: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default, alias = "fallback-models")]
    pub fallback_models: Vec<String>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            url: default_url(),
            model: default_model(),
            fallback_models: Vec::new(),
        }
    }
}

fn default_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_model() -> String {
    "qwen3:4b".to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmSettings {
    #[serde(default = "default_max_tokens", alias = "max-tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_true", alias = "include-lesson-content")]
    pub include_lesson_content: bool,
    #[serde(default = "default_max_history", alias = "max-history-attempts")]
    pub max_history_attempts: u32,
}

impl Default for LlmSettings {
    fn default() -> Self {
        Self {
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
            include_lesson_content: true,
            max_history_attempts: default_max_history(),
        }
    }
}

fn default_max_tokens() -> u32 {
    500
}

fn default_temperature() -> f32 {
    0.3
}

fn default_true() -> bool {
    true
}

fn default_max_history() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_defaults() {
        let config = LlmConfig::default();
        assert_eq!(config.backend, "ollama");
        assert_eq!(config.ollama.url, "http://localhost:11434");
        assert_eq!(config.ollama.model, "qwen3:4b");
        assert_eq!(config.settings.max_tokens, 500);
        assert!((config.settings.temperature - 0.3).abs() < f32::EPSILON);
        assert!(config.settings.include_lesson_content);
        assert_eq!(config.settings.max_history_attempts, 3);
    }

    #[test]
    fn test_llm_config_from_yaml() {
        let yaml = r#"
backend: ollama
ollama:
  url: "http://192.168.1.50:11434"
  model: "qwen3:8b"
  fallback-models: ["llama3:8b"]
settings:
  max-tokens: 800
  temperature: 0.5
"#;
        let config: LlmConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.backend, "ollama");
        assert_eq!(config.ollama.url, "http://192.168.1.50:11434");
        assert_eq!(config.ollama.model, "qwen3:8b");
        assert_eq!(config.ollama.fallback_models, vec!["llama3:8b"]);
        assert_eq!(config.settings.max_tokens, 800);
        assert!((config.settings.temperature - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_llm_config_partial_yaml() {
        let yaml = r#"
backend: ollama
ollama:
  model: "llama3:8b"
"#;
        let config: LlmConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.ollama.url, "http://localhost:11434");
        assert_eq!(config.ollama.model, "llama3:8b");
        assert_eq!(config.settings.max_tokens, 500);
    }

    #[test]
    fn test_llm_config_empty_yaml() {
        let yaml = "{}";
        let config: LlmConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.backend, "ollama");
        assert_eq!(config.ollama.model, "qwen3:4b");
    }
}
