use std::sync::mpsc;

use super::chat::ChatMessage;
use super::context::LlmContext;

/// Request sent from the main TUI thread to the LLM background thread.
pub enum LlmRequest {
    Chat {
        context: LlmContext,
        messages: Vec<ChatMessage>,
    },
    Shutdown,
}

/// Event sent from the LLM background thread back to the main TUI thread.
pub enum LlmEvent {
    /// A single token of streamed output.
    Token(String),
    /// Streaming complete — contains the full assembled response.
    Done(String),
    /// An error occurred during the LLM call.
    Error(String),
    /// Backend is connected and ready.
    BackendReady(String),
    /// Backend is not reachable.
    BackendUnavailable(String),
}

/// Holds both ends of the mpsc channels bridging sync TUI ↔ async LLM thread.
pub struct LlmChannel {
    pub request_tx: mpsc::Sender<LlmRequest>,
    pub response_rx: mpsc::Receiver<LlmEvent>,
}
