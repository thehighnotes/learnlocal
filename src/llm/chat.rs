use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub enum ChatRole {
    System,
    User,
    Assistant,
}

/// In-memory state for the chat panel.
pub struct ChatState {
    /// Full message history for this exercise session.
    pub messages: Vec<ChatMessage>,
    /// The user's current input text.
    pub input_buffer: String,
    /// Buffer accumulating streamed tokens from the current response.
    pub streaming_buffer: String,
    /// Whether we're currently receiving a streamed response.
    pub is_streaming: bool,
    /// Scroll offset for viewing message history.
    pub scroll_offset: u16,
}

impl ChatState {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            input_buffer: String::new(),
            streaming_buffer: String::new(),
            is_streaming: false,
            scroll_offset: 0,
        }
    }

    /// Add a completed assistant message (after streaming finishes).
    pub fn push_assistant_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: ChatRole::Assistant,
            content,
        });
        self.streaming_buffer.clear();
        self.is_streaming = false;
    }

    /// Add a user message.
    pub fn push_user_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: ChatRole::User,
            content,
        });
    }

    /// Append a token to the streaming buffer.
    pub fn append_token(&mut self, token: &str) {
        self.streaming_buffer.push_str(token);
    }

    /// Reset chat for a new exercise.
    pub fn reset(&mut self) {
        self.messages.clear();
        self.input_buffer.clear();
        self.streaming_buffer.clear();
        self.is_streaming = false;
        self.scroll_offset = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_state_lifecycle() {
        let mut state = ChatState::new();
        assert!(state.messages.is_empty());
        assert!(!state.is_streaming);

        state.push_user_message("Why is my code wrong?".to_string());
        assert_eq!(state.messages.len(), 1);
        assert_eq!(state.messages[0].role, ChatRole::User);

        state.is_streaming = true;
        state.append_token("Your code ");
        state.append_token("has a bug.");
        assert_eq!(state.streaming_buffer, "Your code has a bug.");

        state.push_assistant_message("Your code has a bug.".to_string());
        assert_eq!(state.messages.len(), 2);
        assert!(!state.is_streaming);
        assert!(state.streaming_buffer.is_empty());
    }

    #[test]
    fn test_chat_state_reset() {
        let mut state = ChatState::new();
        state.push_user_message("test".to_string());
        state.push_assistant_message("reply".to_string());
        state.input_buffer = "draft".to_string();

        state.reset();
        assert!(state.messages.is_empty());
        assert!(state.input_buffer.is_empty());
        assert!(!state.is_streaming);
    }

    #[test]
    fn test_chat_role_serialization() {
        let msg = ChatMessage {
            role: ChatRole::User,
            content: "hello".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"role\":\"user\""));
    }
}
