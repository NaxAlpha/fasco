//! Chat API request and response types for Cerebras.

use serde::{Deserialize, Serialize};

/// Message role in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System message (sets context/instructions)
    System,
    /// User message
    User,
    /// Assistant message (LLM response)
    Assistant,
}

/// A single message in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The role of the message author
    pub role: Role,
    /// The content of the message
    pub content: String,
}

impl Message {
    /// Create a new system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    /// Create a new user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    /// Create a new assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// Chat completion request.
#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    /// The model to use
    pub model: String,
    /// The conversation messages
    pub messages: Vec<Message>,
    /// Whether to stream the response
    /// Whether to stream the response
    #[serde(skip_serializing_if = "is_false")]
    pub stream: bool,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(val: &bool) -> bool {
    !val
}

/// A single choice in a chat completion response.
#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    /// The reason the model stopped generating
    pub finish_reason: Option<String>,
    /// The message content
    pub message: Message,
}

/// Usage information for the request.
#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    /// Input tokens used
    pub prompt_tokens: u32,
    /// Output tokens generated
    pub completion_tokens: u32,
    /// Total tokens used
    pub total_tokens: u32,
}

/// Chat completion response.
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponse {
    /// Unique identifier for the response
    pub id: String,
    /// The object type (always "chat.completion")
    pub object: String,
    /// When the response was created
    pub created: u64,
    /// The model used
    pub model: String,
    /// The choices (typically one)
    pub choices: Vec<Choice>,
    /// Token usage information
    pub usage: Option<Usage>,
}

/// A delta (partial content) in a streaming response.
#[derive(Debug, Clone, Deserialize)]
pub struct Delta {
    /// The role (only in first chunk)
    pub role: Option<Role>,
    /// Content delta
    pub content: Option<String>,
}

/// A choice in a streaming response.
#[derive(Debug, Clone, Deserialize)]
pub struct StreamChoice {
    /// The index of this choice
    pub index: u32,
    /// The delta content
    pub delta: Delta,
    /// Finish reason (only in final chunk)
    pub finish_reason: Option<String>,
}

/// Streaming chat completion chunk.
#[derive(Debug, Clone, Deserialize)]
pub struct StreamChunk {
    /// Unique identifier for the chunk
    pub id: String,
    /// The object type (always "chat.completion.chunk")
    pub object: String,
    /// When the chunk was created
    pub created: u64,
    /// The model being used
    pub model: String,
    /// The choices (typically one)
    pub choices: Vec<StreamChoice>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let sys = Message::system("You are helpful.");
        assert_eq!(sys.role, Role::System);
        assert_eq!(sys.content, "You are helpful.");

        let user = Message::user("Hello");
        assert_eq!(user.role, Role::User);
        assert_eq!(user.content, "Hello");

        let asst = Message::assistant("Hi there!");
        assert_eq!(asst.role, Role::Assistant);
        assert_eq!(asst.content, "Hi there!");
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::user("test");
        let json = serde_json::to_string(&msg).unwrap();
        assert_eq!(json, r#"{"role":"user","content":"test"}"#);
    }

    #[test]
    fn test_message_deserialization() {
        let json = r#"{"role":"assistant","content":"response"}"#;
        let msg: Message = serde_json::from_str(json).unwrap();
        assert_eq!(msg.role, Role::Assistant);
        assert_eq!(msg.content, "response");
    }

    #[test]
    fn test_chat_request_serialization() {
        let req = ChatRequest {
            model: "zai-glm-4.7".to_string(),
            messages: vec![Message::user("hello")],
            stream: false,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains(r#""model":"zai-glm-4.7""#));
        assert!(json.contains(r#""messages""#));
        // stream: false should be omitted
        assert!(!json.contains("stream"));
    }

    #[test]
    fn test_chat_request_stream_serialization() {
        let req = ChatRequest {
            model: "zai-glm-4.7".to_string(),
            messages: vec![Message::user("hello")],
            stream: true,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains(r#""stream":true"#));
    }
}
