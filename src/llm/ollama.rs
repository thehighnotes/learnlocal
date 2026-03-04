use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

use super::backend::LlmBackend;
use super::channel::{LlmChannel, LlmEvent, LlmRequest};
use super::chat::{ChatMessage, ChatRole};
use super::config::LlmConfig;
use super::context::LlmContext;

/// Fetch available model names from an Ollama instance.
/// Standalone function — does not require an OllamaBackend.
pub async fn list_available_models(base_url: &str) -> Result<Vec<String>, String> {
    let url = format!("{}/api/tags", base_url.trim_end_matches('/'));
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to Ollama at {}: {}", base_url, e))?;

    if !resp.status().is_success() {
        return Err(format!("Ollama returned status {}", resp.status()));
    }

    let body: OllamaTagsResponse = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

    Ok(body.models.into_iter().map(|m| m.name).collect())
}

pub struct OllamaBackend {
    client: Client,
    base_url: String,
    model: String,
    fallback_models: Vec<String>,
    max_tokens: u32,
    temperature: f32,
}

impl OllamaBackend {
    pub fn new(config: &LlmConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap_or_default();

        Self {
            client,
            base_url: config.ollama.url.trim_end_matches('/').to_string(),
            model: config.ollama.model.clone(),
            fallback_models: config.ollama.fallback_models.clone(),
            max_tokens: config.settings.max_tokens,
            temperature: config.settings.temperature,
        }
    }

    /// Try to find a working model from primary + fallbacks.
    async fn find_available_model(&self) -> Option<String> {
        let models = match self.list_models().await {
            Ok(m) => m,
            Err(_) => return None,
        };

        // Check primary model
        if models.iter().any(|m| m == &self.model) {
            return Some(self.model.clone());
        }

        // Check fallbacks
        for fallback in &self.fallback_models {
            if models.iter().any(|m| m == fallback) {
                return Some(fallback.clone());
            }
        }

        None
    }

    async fn list_models(&self) -> Result<Vec<String>, String> {
        let url = format!("{}/api/tags", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Ollama returned status {}", resp.status()));
        }

        let body: OllamaTagsResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        Ok(body.models.into_iter().map(|m| m.name).collect())
    }

    async fn do_chat_stream(
        &self,
        model: &str,
        context: &LlmContext,
        messages: &[ChatMessage],
        token_tx: &mpsc::Sender<LlmEvent>,
    ) -> Result<(), String> {
        let system_prompt = context.to_system_prompt();

        let mut api_messages = vec![OllamaChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        }];

        for msg in messages {
            api_messages.push(OllamaChatMessage {
                role: match msg.role {
                    ChatRole::System => "system".to_string(),
                    ChatRole::User => "user".to_string(),
                    ChatRole::Assistant => "assistant".to_string(),
                },
                content: msg.content.clone(),
            });
        }

        let body = OllamaChatRequest {
            model: model.to_string(),
            messages: api_messages,
            stream: true,
            options: OllamaChatOptions {
                num_predict: self.max_tokens as i32,
                temperature: self.temperature,
            },
        };

        let url = format!("{}/api/chat", self.base_url);
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Ollama request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Ollama returned {}: {}", status, text));
        }

        // Stream response: Ollama sends newline-delimited JSON chunks
        let mut full_text = String::new();
        let mut line_buffer = String::new();
        let mut stream = resp.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| format!("Failed to read Ollama stream: {}", e))?;
            line_buffer.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete lines (Ollama sends newline-delimited JSON)
            while let Some(newline_pos) = line_buffer.find('\n') {
                let line = line_buffer[..newline_pos].to_string();
                line_buffer = line_buffer[newline_pos + 1..].to_string();

                if line.trim().is_empty() {
                    continue;
                }

                match serde_json::from_str::<OllamaChatChunk>(&line) {
                    Ok(chunk) => {
                        let token = chunk.message.content;
                        if !token.is_empty() {
                            full_text.push_str(&token);
                            let _ = token_tx.send(LlmEvent::Token(token));
                        }
                        if chunk.done {
                            let _ = token_tx.send(LlmEvent::Done(full_text));
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        return Err(format!("Failed to parse chunk: {}", e));
                    }
                }
            }
        }

        // If stream ended without done=true, send what we have
        let _ = token_tx.send(LlmEvent::Done(full_text));
        Ok(())
    }
}

impl LlmBackend for OllamaBackend {
    fn is_available(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(async move { self.find_available_model().await.is_some() })
    }

    fn chat_stream(
        &self,
        context: &LlmContext,
        messages: &[ChatMessage],
        token_tx: mpsc::Sender<LlmEvent>,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
        let messages = messages.to_vec();
        let context = context.clone();
        Box::pin(async move {
            let model = match self.find_available_model().await {
                Some(m) => m,
                None => {
                    let _ = token_tx.send(LlmEvent::Error("No model available".to_string()));
                    return Err("No model available".to_string());
                }
            };
            self.do_chat_stream(&model, &context, &messages, &token_tx)
                .await
        })
    }

    fn name(&self) -> String {
        format!("Ollama ({})", self.model)
    }
}

/// Spawn the LLM background thread with a tokio current_thread runtime inside.
pub fn spawn_llm_thread(config: LlmConfig) -> LlmChannel {
    let (req_tx, req_rx) = mpsc::channel::<LlmRequest>();
    let (resp_tx, resp_rx) = mpsc::channel::<LlmEvent>();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime for LLM thread");

        rt.block_on(async move {
            let backend = OllamaBackend::new(&config);

            // Check availability
            if let Some(model) = backend.find_available_model().await {
                let _ = resp_tx.send(LlmEvent::BackendReady(format!("Ollama ({})", model)));
            } else {
                let _ = resp_tx.send(LlmEvent::BackendUnavailable(format!(
                    "Cannot reach Ollama at {} or model {} not found",
                    config.ollama.url, config.ollama.model
                )));
                return;
            }

            // Event loop: receive requests, dispatch to backend
            while let Ok(req) = req_rx.recv() {
                match req {
                    LlmRequest::Chat { context, messages } => {
                        if let Err(e) = backend
                            .chat_stream(&context, &messages, resp_tx.clone())
                            .await
                        {
                            let _ = resp_tx.send(LlmEvent::Error(e));
                        }
                    }
                    LlmRequest::Shutdown => break,
                }
            }
        });
    });

    LlmChannel {
        request_tx: req_tx,
        response_rx: resp_rx,
    }
}

// --- Ollama API types ---

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct OllamaModel {
    name: String,
}

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaChatMessage>,
    stream: bool,
    options: OllamaChatOptions,
}

#[derive(Serialize)]
struct OllamaChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OllamaChatOptions {
    num_predict: i32,
    temperature: f32,
}

#[derive(Deserialize)]
struct OllamaChatChunk {
    message: OllamaChatChunkMessage,
    #[serde(default)]
    done: bool,
}

#[derive(Deserialize)]
struct OllamaChatChunkMessage {
    #[serde(default)]
    content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tags_response() {
        let json =
            r#"{"models":[{"name":"qwen3:4b","size":12345},{"name":"llama3:8b","size":67890}]}"#;
        let resp: OllamaTagsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.models.len(), 2);
        assert_eq!(resp.models[0].name, "qwen3:4b");
        assert_eq!(resp.models[1].name, "llama3:8b");
    }

    #[test]
    fn test_parse_chat_chunk() {
        let json =
            r#"{"model":"qwen3:4b","message":{"role":"assistant","content":"Hello"},"done":false}"#;
        let chunk: OllamaChatChunk = serde_json::from_str(json).unwrap();
        assert_eq!(chunk.message.content, "Hello");
        assert!(!chunk.done);
    }

    #[test]
    fn test_parse_chat_chunk_done() {
        let json =
            r#"{"model":"qwen3:4b","message":{"role":"assistant","content":""},"done":true}"#;
        let chunk: OllamaChatChunk = serde_json::from_str(json).unwrap();
        assert!(chunk.done);
        assert!(chunk.message.content.is_empty());
    }

    #[test]
    fn test_chat_request_serialization() {
        let req = OllamaChatRequest {
            model: "qwen3:4b".to_string(),
            messages: vec![
                OllamaChatMessage {
                    role: "system".to_string(),
                    content: "You are a tutor.".to_string(),
                },
                OllamaChatMessage {
                    role: "user".to_string(),
                    content: "Help me".to_string(),
                },
            ],
            stream: true,
            options: OllamaChatOptions {
                num_predict: 500,
                temperature: 0.3,
            },
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"model\":\"qwen3:4b\""));
        assert!(json.contains("\"stream\":true"));
        assert!(json.contains("\"num_predict\":500"));
    }

    #[test]
    fn test_ollama_backend_name() {
        let config = LlmConfig::default();
        let backend = OllamaBackend::new(&config);
        assert_eq!(backend.name(), "Ollama (qwen3:4b)");
    }
}
