//! Integration tests for Fasco.

use fasco::models::{ChatRequest, Choice, Message, Role, StreamChunk};
use serde_json::json;

#[test]
fn test_message_role_serialization() {
    // System role
    let msg = Message::system("test");
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains(r#""role":"system""#));

    // User role
    let msg = Message::user("test");
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains(r#""role":"user""#));

    // Assistant role
    let msg = Message::assistant("test");
    let json = serde_json::to_string(&msg).unwrap();
    assert!(json.contains(r#""role":"assistant""#));
}

#[test]
fn test_message_role_deserialization() {
    // System
    let json = json!({"role": "system", "content": "be helpful"});
    let msg: Message = serde_json::from_value(json).unwrap();
    assert_eq!(msg.role, Role::System);

    // User
    let json = json!({"role": "user", "content": "hello"});
    let msg: Message = serde_json::from_value(json).unwrap();
    assert_eq!(msg.role, Role::User);

    // Assistant
    let json = json!({"role": "assistant", "content": "hi"});
    let msg: Message = serde_json::from_value(json).unwrap();
    assert_eq!(msg.role, Role::Assistant);
}

#[test]
fn test_message_with_special_characters() {
    let msg = Message::user("Hello \"world\"\nNew line\tTab");
    let json = serde_json::to_string(&msg).unwrap();
    let deserialized: Message = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.content, "Hello \"world\"\nNew line\tTab");
}

#[test]
fn test_chat_request_serialization() {
    let req = ChatRequest {
        model: "zai-glm-4.7".to_string(),
        messages: vec![Message::system("You are helpful."), Message::user("Hello!")],
        stream: false,
    };

    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains(r#""model":"zai-glm-4.7""#));
    assert!(json.contains(r#""role":"system""#));
    assert!(json.contains(r#""role":"user""#));
    assert!(!json.contains("stream")); // false should be omitted
}

#[test]
fn test_chat_request_with_stream() {
    let req = ChatRequest {
        model: "zai-glm-4.7".to_string(),
        messages: vec![Message::user("test")],
        stream: true,
    };

    let json = serde_json::to_string(&req).unwrap();
    assert!(json.contains(r#""stream":true"#));
}

#[test]
fn test_stream_chunk_parsing() {
    let json = r#"{
        "id": "chunk-123",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "zai-glm-4.7",
        "choices": [{
            "index": 0,
            "delta": {
                "content": "Hello"
            }
        }]
    }"#;

    let chunk: StreamChunk = serde_json::from_str(json).unwrap();
    assert_eq!(chunk.id, "chunk-123");
    assert_eq!(chunk.model, "zai-glm-4.7");
    assert!(!chunk.choices.is_empty());
    assert_eq!(chunk.choices[0].delta.content.as_ref().unwrap(), "Hello");
}

#[test]
fn test_stream_chunk_with_finish_reason() {
    let json = r#"{
        "id": "chunk-final",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "zai-glm-4.7",
        "choices": [{
            "index": 0,
            "delta": {},
            "finish_reason": "stop"
        }]
    }"#;

    let chunk: StreamChunk = serde_json::from_str(json).unwrap();
    assert_eq!(chunk.choices[0].finish_reason.as_ref().unwrap(), "stop");
}

#[test]
fn test_stream_chunk_with_role() {
    let json = r#"{
        "id": "chunk-first",
        "object": "chat.completion.chunk",
        "created": 1234567890,
        "model": "zai-glm-4.7",
        "choices": [{
            "index": 0,
            "delta": {
                "role": "assistant"
            }
        }]
    }"#;

    let chunk: StreamChunk = serde_json::from_str(json).unwrap();
    assert_eq!(
        chunk.choices[0].delta.role.as_ref().unwrap(),
        &Role::Assistant
    );
}

#[test]
fn test_choice_parsing() {
    let json = r#"{
        "finish_reason": "stop",
        "message": {
            "role": "assistant",
            "content": "Hello, how can I help?"
        }
    }"#;

    let choice: Choice = serde_json::from_str(json).unwrap();
    assert_eq!(choice.finish_reason.unwrap(), "stop");
    assert_eq!(choice.message.role, Role::Assistant);
    assert_eq!(choice.message.content, "Hello, how can I help?");
}

#[tokio::test]
async fn test_client_headers() {
    let client = fasco::client::CerebrasClient::new("sk-test-key-123");
    let headers = client.build_headers();

    assert_eq!(
        headers.get("content-type").unwrap().to_str().unwrap(),
        "application/json"
    );
    assert_eq!(
        headers.get("authorization").unwrap().to_str().unwrap(),
        "Bearer sk-test-key-123"
    );
}

#[tokio::test]
async fn test_client_with_custom_model() {
    let client = fasco::client::CerebrasClient::new("test-key").with_model("custom-model-name");
    assert_eq!(client.model(), "custom-model-name");
}

#[tokio::test]
async fn test_client_with_custom_base_url() {
    let client = fasco::client::CerebrasClient::with_base_url(
        "test-key",
        "https://custom.api.example.com/v1",
    );
    assert_eq!(client.base_url(), "https://custom.api.example.com/v1");
}

#[test]
fn test_message_convenience_methods() {
    let sys = Message::system("system prompt");
    assert_eq!(sys.role, Role::System);
    assert_eq!(sys.content, "system prompt");

    let user = Message::user("user message");
    assert_eq!(user.role, Role::User);
    assert_eq!(user.content, "user message");

    let asst = Message::assistant("assistant message");
    assert_eq!(asst.role, Role::Assistant);
    assert_eq!(asst.content, "assistant message");
}
