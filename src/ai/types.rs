use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum AiProvider {
    #[default]
    OpenAi,
    Anthropic,
    Gemini,
    OpenRouter,
    Ollama,
    Custom,
}

impl std::fmt::Display for AiProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProvider::OpenAi => write!(f, "OpenAI"),
            AiProvider::Anthropic => write!(f, "Anthropic"),
            AiProvider::Gemini => write!(f, "Google Gemini"),
            AiProvider::OpenRouter => write!(f, "OpenRouter"),
            AiProvider::Ollama => write!(f, "Ollama"),
            AiProvider::Custom => write!(f, "Custom"),
        }
    }
}

impl AiProvider {
    pub fn all() -> &'static [AiProvider] {
        &[
            AiProvider::OpenAi,
            AiProvider::Anthropic,
            AiProvider::Gemini,
            AiProvider::OpenRouter,
            AiProvider::Ollama,
            AiProvider::Custom,
        ]
    }

    pub fn default_base_url(&self) -> &str {
        match self {
            AiProvider::OpenAi => "https://api.openai.com/v1",
            AiProvider::Anthropic => "https://api.anthropic.com/v1",
            AiProvider::Gemini => "https://generativelanguage.googleapis.com/v1beta",
            AiProvider::OpenRouter => "https://openrouter.ai/api/v1",
            AiProvider::Ollama => "http://localhost:11434",
            AiProvider::Custom => "",
        }
    }

    pub fn default_model(&self) -> &str {
        match self {
            AiProvider::OpenAi => "gpt-4o",
            AiProvider::Anthropic => "claude-sonnet-4-20250514",
            AiProvider::Gemini => "gemini-2.0-flash",
            AiProvider::OpenRouter => "openai/gpt-4o",
            AiProvider::Ollama => "llama3.1",
            AiProvider::Custom => "",
        }
    }

    pub fn known_models(&self) -> &'static [&'static str] {
        match self {
            AiProvider::OpenAi => &[
                "gpt-4o",
                "gpt-4o-mini",
                "gpt-4-turbo",
                "gpt-4",
                "gpt-3.5-turbo",
                "o1",
                "o1-mini",
                "o1-pro",
            ],
            AiProvider::Anthropic => &[
                "claude-sonnet-4-20250514",
                "claude-3-5-sonnet-20241022",
                "claude-3-5-haiku-20241022",
                "claude-3-opus-20240229",
                "claude-3-haiku-20240307",
            ],
            AiProvider::Gemini => &[
                "gemini-2.0-flash",
                "gemini-2.0-flash-lite",
                "gemini-1.5-pro",
                "gemini-1.5-flash",
                "gemini-1.5-flash-8b",
            ],
            AiProvider::OpenRouter => &[
                "openai/gpt-4o",
                "openai/gpt-4o-mini",
                "anthropic/claude-sonnet-4-20250514",
                "anthropic/claude-3.5-sonnet",
                "google/gemini-2.0-flash-001",
                "meta-llama/llama-3.1-405b-instruct",
                "mistralai/mixtral-8x7b-instruct",
                "deepseek/deepseek-chat",
            ],
            AiProvider::Ollama => &[
                "llama3.1",
                "llama3.1:8b",
                "llama3.1:70b",
                "codellama",
                "mistral",
                "mixtral",
                "phi3",
                "gemma2",
                "qwen2.5",
            ],
            AiProvider::Custom => &[],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    pub id: Option<i32>,
    pub provider: AiProvider,
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub is_default: bool,
}

impl Default for AiProviderConfig {
    fn default() -> Self {
        Self {
            id: None,
            provider: AiProvider::OpenAi,
            name: "OpenAI".to_string(),
            base_url: AiProvider::OpenAi.default_base_url().to_string(),
            model: AiProvider::OpenAi.default_model().to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            is_default: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AiRole {
    System,
    User,
    Assistant,
}

impl std::fmt::Display for AiRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiRole::System => write!(f, "system"),
            AiRole::User => write!(f, "user"),
            AiRole::Assistant => write!(f, "assistant"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatMessage {
    pub role: AiRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiChatRequest {
    pub model: String,
    pub messages: Vec<AiChatMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AiChatResponse {
    pub content: String,
    pub model: String,
    pub usage: AiUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[allow(dead_code)]
pub struct AiUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct AiProviderError {
    pub message: String,
    pub code: Option<String>,
}

impl std::fmt::Display for AiProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.code {
            Some(code) => write!(f, "[{}] {}", code, self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for AiProviderError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AiConversation {
    pub id: Option<i32>,
    pub provider_config_id: i32,
    pub title: Option<String>,
    pub system_prompt: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AiMessage {
    pub id: Option<i32>,
    pub conversation_id: i32,
    pub role: AiRole,
    pub content: String,
    pub tokens_used: u32,
    pub created_at: String,
}
