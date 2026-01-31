//! Cerebras API client for chat completions with streaming support.

use anyhow::{Context, Result};
use futures::stream::{Stream, StreamExt};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use tracing::{debug, info};

use crate::models::{ChatRequest, Message, StreamChunk};

/// Default Cerebras API base URL.
pub const DEFAULT_BASE_URL: &str = "https://api.cerebras.ai/v1";

/// Default model to use.
pub const DEFAULT_MODEL: &str = "zai-glm-4.7";

/// Cerebras API client for chat completions.
pub struct CerebrasClient {
    /// HTTP client for making requests
    http: reqwest::Client,
    /// API key for authentication
    api_key: String,
    /// Base URL for the API
    base_url: String,
    /// Model to use for requests
    model: String,
}

impl CerebrasClient {
    /// Create a new Cerebras client.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The Cerebras API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_base_url(api_key, DEFAULT_BASE_URL)
    }

    /// Create a new Cerebras client with a custom base URL.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The Cerebras API key
    /// * `base_url` - Custom base URL for the API
    pub fn with_base_url(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key: api_key.into(),
            base_url: base_url.into(),
            model: DEFAULT_MODEL.to_string(),
        }
    }

    /// Set the model to use for requests.
    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Get the current model.
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Get the base URL.
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Build the request headers.
    ///
    /// # Panics
    ///
    /// Panics if the API key contains invalid characters for an HTTP header value.
    pub fn build_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .expect("invalid authorization header value"),
        );
        headers
    }

    /// Send a chat completion request with streaming response.
    ///
    /// Returns a stream of content deltas as they arrive from the API.
    ///
    /// # Arguments
    ///
    /// * `messages` - The conversation history including the new user message
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the API returns an error response.
    pub async fn chat_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<impl Stream<Item = Result<String>>> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            stream: true,
        };

        debug!("sending chat request to {}", self.base_url);

        let url = format!("{}/chat/completions", self.base_url);
        let response = self
            .http
            .post(&url)
            .headers(self.build_headers())
            .json(&request)
            .send()
            .await
            .context("failed to send request to Cerebras API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Cerebras API returned error {status}: {error_text}");
        }

        info!("chat request successful, streaming response");

        // Parse Server-Sent Events (SSE) from the response body
        let content_stream = async_stream::try_stream! {
            let mut response_stream = response.bytes_stream();
            let mut buffer = Vec::new();

            while let Some(chunk_result) = response_stream.next().await {
                let chunk = chunk_result.context("failed to read chunk")?;
                buffer.extend_from_slice(&chunk);

                // Process all complete lines in the buffer
                while let Some(newline_pos) = buffer.iter().position(|&b| b == b'\n') {
                    let drained: Vec<u8> = buffer.drain(..=newline_pos).collect();
                    let line = String::from_utf8_lossy(&drained);
                    let line = line.trim();

                    if line.is_empty() {
                        continue;
                    }

                    if line == "data: [DONE]" {
                        debug!("received [DONE] from stream");
                        break;
                    }

                    if let Some(data) = line.strip_prefix("data: ") {
                        match serde_json::from_str::<StreamChunk>(data) {
                            Ok(chunk) => {
                                if let Some(choice) = chunk.choices.first()
                                    && let Some(content) = &choice.delta.content
                                {
                                    debug!("yielding content delta: {content}");
                                    yield content.clone();
                                }
                            }
                            Err(e) => {
                                debug!("failed to parse chunk: {e}");
                            }
                        }
                    }
                }
            }
        };

        Ok(content_stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = CerebrasClient::new("test-key");
        assert_eq!(client.api_key, "test-key");
        assert_eq!(client.base_url, DEFAULT_BASE_URL);
        assert_eq!(client.model, DEFAULT_MODEL);
    }

    #[test]
    fn test_client_with_base_url() {
        let client = CerebrasClient::with_base_url("test-key", "https://custom.api/v1");
        assert_eq!(client.api_key, "test-key");
        assert_eq!(client.base_url, "https://custom.api/v1");
    }

    #[test]
    fn test_client_with_model() {
        let client = CerebrasClient::new("test-key").with_model("custom-model");
        assert_eq!(client.model, "custom-model");
    }

    #[test]
    fn test_build_headers() {
        let client = CerebrasClient::new("sk-test-key-123");
        let headers = client.build_headers();

        assert_eq!(headers.get(CONTENT_TYPE).unwrap(), "application/json");
        assert_eq!(
            headers.get(AUTHORIZATION).unwrap(),
            "Bearer sk-test-key-123"
        );
    }
}
