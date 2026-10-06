use crate::ai::provider::AiProviderAdapter;
use crate::ai::types::{AiChatRequest, AiChatResponse, AiProviderError, AiRole, AiUsage};
use async_trait::async_trait;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

pub struct AnthropicProvider {
    pub api_key: String,
    pub base_url: String,
    pub http_client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            http_client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct MessagesRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    stream: bool,
}

#[derive(Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct MessagesResponse {
    content: Vec<ContentBlock>,
    usage: Option<AnthropicUsage>,
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicUsage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct AnthropicError {
    error: AnthropicErrorDetail,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct AnthropicErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: Option<String>,
}

// Streaming types

#[derive(Deserialize)]
#[allow(dead_code)]
struct StreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    delta: Option<StreamDelta>,
    usage: Option<AnthropicUsage>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct StreamDelta {
    text: Option<String>,
    partial_json: Option<String>,
}

#[async_trait]
impl AiProviderAdapter for AnthropicProvider {
    async fn chat(&self, request: AiChatRequest) -> Result<AiChatResponse, AiProviderError> {
        let (system, messages) = split_system_prompt(&request);

        let body = MessagesRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            system,
            stream: false,
        };

        let response = self
            .http_client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AiProviderError {
                message: format!("Request failed: {e}"),
                code: None,
            })?;

        let status = response.status();
        let text = response.text().await.map_err(|e| AiProviderError {
            message: format!("Failed to read response: {e}"),
            code: None,
        })?;

        if !status.is_success() {
            let api_err: Result<AnthropicError, _> = serde_json::from_str(&text);
            return Err(AiProviderError {
                message: api_err
                    .as_ref()
                    .map(|e| e.error.message.clone())
                    .unwrap_or_else(|_| text.clone()),
                code: Some(status.as_u16().to_string()),
            });
        }

        let resp: MessagesResponse =
            serde_json::from_str(&text).map_err(|e| AiProviderError {
                message: format!("Failed to parse response: {e}"),
                code: None,
            })?;

        let content = resp
            .content
            .into_iter()
            .filter(|b| b.block_type == "text")
            .filter_map(|b| b.text)
            .collect::<Vec<_>>()
            .join("");

        let usage = resp.usage.map(|u| AiUsage {
            prompt_tokens: u.input_tokens.unwrap_or(0),
            completion_tokens: u.output_tokens.unwrap_or(0),
            total_tokens: u
                .input_tokens
                .unwrap_or(0)
                .saturating_add(u.output_tokens.unwrap_or(0)),
        }).unwrap_or_default();

        Ok(AiChatResponse {
            content,
            model: body.model,
            usage,
        })
    }

    async fn chat_stream(
        &self,
        request: AiChatRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<String>, AiProviderError> {
        let (system, messages) = split_system_prompt(&request);

        let body = MessagesRequest {
            model: request.model.clone(),
            messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            system,
            stream: true,
        };

        let response = self
            .http_client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AiProviderError {
                message: format!("Request failed: {e}"),
                code: None,
            })?;

        let status = response.status();
        if !status.is_success() {
            let text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let api_err: Result<AnthropicError, _> = serde_json::from_str(&text);
            return Err(AiProviderError {
                message: api_err
                    .as_ref()
                    .map(|e| e.error.message.clone())
                    .unwrap_or_else(|_| text.clone()),
                code: Some(status.as_u16().to_string()),
            });
        }

        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let stream = response.bytes_stream();

        tokio::spawn(async move {
            let mut buffer = String::new();
            let mut stream = stream;

            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(e) => {
                        log::warn!("Anthropic stream chunk error: {e}");
                        break;
                    }
                };

                buffer.push_str(&String::from_utf8_lossy(&chunk));

                while let Some(newline_pos) = buffer.find('\n') {
                    let line = buffer[..newline_pos].trim().to_string();
                    buffer = buffer[newline_pos + 1..].to_string();

                    if line.is_empty() || !line.starts_with("data: ") {
                        continue;
                    }

                    let data = &line[6..];

                    if let Ok(event) = serde_json::from_str::<StreamEvent>(data) {
                        match event.event_type.as_str() {
                            "content_block_delta" => {
                                if let Some(delta) = event.delta {
                                    if let Some(text) = delta.text {
                                        if tx.send(text).await.is_err() {
                                            return;
                                        }
                                    }
                                }
                            }
                            "message_stop" => {
                                return;
                            }
                            "error" => {
                                log::warn!("Anthropic stream error event: {data}");
                                return;
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        Ok(rx)
    }

    fn name(&self) -> &str {
        "Anthropic"
    }

    fn available_models(&self) -> Vec<&str> {
        vec![
            "claude-sonnet-4-20250514",
            "claude-3-5-sonnet-20241022",
            "claude-3-5-haiku-20241022",
            "claude-3-opus-20240229",
            "claude-3-haiku-20240307",
        ]
    }
}

fn split_system_prompt(
    request: &AiChatRequest,
) -> (Option<String>, Vec<AnthropicMessage>) {
    let mut system = None;
    let mut messages = Vec::new();

    for msg in &request.messages {
        match msg.role {
            AiRole::System => {
                // Anthropic takes system as a top-level field
                if system.is_some() {
                    // Concatenate multiple system messages
                    let existing = system.take().unwrap_or_default();
                    system = Some(format!("{existing}\n\n{}", msg.content));
                } else {
                    system = Some(msg.content.clone());
                }
            }
            AiRole::User => {
                messages.push(AnthropicMessage {
                    role: "user".to_string(),
                    content: msg.content.clone(),
                });
            }
            AiRole::Assistant => {
                messages.push(AnthropicMessage {
                    role: "assistant".to_string(),
                    content: msg.content.clone(),
                });
            }
        }
    }

    (system, messages)
}
