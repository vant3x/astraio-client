use crate::ai::provider::AiProviderAdapter;
use crate::ai::types::{AiChatRequest, AiChatResponse, AiProviderError, AiRole, AiUsage};
use async_trait::async_trait;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

pub struct GeminiProvider {
    pub api_key: String,
    pub base_url: String,
    pub http_client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new(api_key: String, base_url: String) -> Self {
        Self {
            api_key,
            base_url,
            http_client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GenerationConfig>,
}

#[derive(Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize)]
struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    usage_metadata: Option<GeminiUsage>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct GeminiCandidate {
    content: Option<GeminiContent>,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct GeminiUsage {
    prompt_token_count: Option<u32>,
    candidates_token_count: Option<u32>,
    total_token_count: Option<u32>,
}

#[derive(Deserialize)]
struct GeminiError {
    error: GeminiErrorDetail,
}

#[derive(Deserialize)]
struct GeminiErrorDetail {
    message: String,
    #[serde(rename = "code")]
    code: Option<u16>,
}

// Streaming

#[derive(Deserialize)]
struct GeminiStreamChunk {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[async_trait]
impl AiProviderAdapter for GeminiProvider {
    async fn chat(&self, request: AiChatRequest) -> Result<AiChatResponse, AiProviderError> {
        let (system, contents) = split_system_prompt(&request);

        let body = GeminiRequest {
            contents,
            system_instruction: system,
            generation_config: Some(GenerationConfig {
                max_output_tokens: request.max_tokens,
                temperature: request.temperature,
            }),
        };

        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, request.model, self.api_key
        );

        let response = self
            .http_client
            .post(&url)
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
            let api_err: Result<GeminiError, _> = serde_json::from_str(&text);
            let code = api_err
                .as_ref()
                .ok()
                .and_then(|e| e.error.code)
                .map(|c| c.to_string())
                .or_else(|| Some(status.as_u16().to_string()));
            return Err(AiProviderError {
                message: api_err
                    .as_ref()
                    .ok()
                    .map(|e| e.error.message.clone())
                    .unwrap_or_else(|| text.clone()),
                code,
            });
        }

        let resp: GeminiResponse =
            serde_json::from_str(&text).map_err(|e| AiProviderError {
                message: format!("Failed to parse response: {e}"),
                code: None,
            })?;

        let content = resp
            .candidates
            .as_ref()
            .and_then(|c| c.first())
            .and_then(|c| c.content.as_ref())
            .map(|c| {
                c.parts
                    .iter()
                    .map(|p| p.text.as_str())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default();

        let usage = resp.usage_metadata.map(|u| AiUsage {
            prompt_tokens: u.prompt_token_count.unwrap_or(0),
            completion_tokens: u.candidates_token_count.unwrap_or(0),
            total_tokens: u.total_token_count.unwrap_or(0),
        }).unwrap_or_default();

        Ok(AiChatResponse {
            content,
            model: request.model,
            usage,
        })
    }

    async fn chat_stream(
        &self,
        request: AiChatRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<String>, AiProviderError> {
        let (system, contents) = split_system_prompt(&request);

        let body = GeminiRequest {
            contents,
            system_instruction: system,
            generation_config: Some(GenerationConfig {
                max_output_tokens: request.max_tokens,
                temperature: request.temperature,
            }),
        };

        let url = format!(
            "{}/models/{}:streamGenerateContent?alt=sse&key={}",
            self.base_url, request.model, self.api_key
        );

        let response = self
            .http_client
            .post(&url)
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
            let api_err: Result<GeminiError, _> = serde_json::from_str(&text);
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
                        log::warn!("Gemini stream chunk error: {e}");
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

                    if let Ok(chunk) = serde_json::from_str::<GeminiStreamChunk>(data) {
                        if let Some(candidates) = chunk.candidates {
                            if let Some(candidate) = candidates.first() {
                                if let Some(content) = &candidate.content {
                                    for part in &content.parts {
                                        if tx.send(part.text.clone()).await.is_err() {
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(rx)
    }

    fn name(&self) -> &str {
        "Gemini"
    }

    fn available_models(&self) -> Vec<&str> {
        vec![
            "gemini-2.0-flash",
            "gemini-2.0-flash-lite",
            "gemini-1.5-pro",
            "gemini-1.5-flash",
            "gemini-1.5-flash-8b",
        ]
    }
}

fn split_system_prompt(
    request: &AiChatRequest,
) -> (Option<GeminiContent>, Vec<GeminiContent>) {
    let mut system = None;
    let mut contents = Vec::new();

    for msg in &request.messages {
        match msg.role {
            AiRole::System => {
                if system.is_none() {
                    system = Some(GeminiContent {
                        role: "user".to_string(),
                        parts: vec![GeminiPart {
                            text: msg.content.clone(),
                        }],
                    });
                }
            }
            AiRole::User => {
                contents.push(GeminiContent {
                    role: "user".to_string(),
                    parts: vec![GeminiPart {
                        text: msg.content.clone(),
                    }],
                });
            }
            AiRole::Assistant => {
                contents.push(GeminiContent {
                    role: "model".to_string(),
                    parts: vec![GeminiPart {
                        text: msg.content.clone(),
                    }],
                });
            }
        }
    }

    (system, contents)
}
