use std::future::Future;
use std::pin::Pin;
use std::sync::mpsc;

use super::channel::LlmEvent;
use super::chat::ChatMessage;
use super::context::LlmContext;

/// Trait for LLM backends. Using Pin<Box<...>> for object safety (no async-trait crate needed).
#[allow(dead_code)]
pub trait LlmBackend: Send {
    /// Check if the backend is reachable and the configured model is available.
    fn is_available(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>>;

    /// Send a chat request with streaming. Tokens are sent through token_tx.
    fn chat_stream(
        &self,
        context: &LlmContext,
        messages: &[ChatMessage],
        token_tx: mpsc::Sender<LlmEvent>,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>>;

    /// The display name of this backend (e.g. "Ollama (llama3:8b)").
    fn name(&self) -> String;
}
