// LLM integration — behind --features llm

#[cfg(feature = "llm")]
pub mod backend;
#[cfg(feature = "llm")]
pub mod channel;
#[cfg(feature = "llm")]
pub mod chat;
#[cfg(feature = "llm")]
pub mod config;
#[cfg(feature = "llm")]
pub mod context;
#[cfg(feature = "llm")]
pub mod ollama;
