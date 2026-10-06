use super::provider::AiProviderAdapter;
use super::providers::anthropic::AnthropicProvider;
use super::providers::gemini::GeminiProvider;
use super::providers::ollama::OllamaProvider;
use super::providers::openai::OpenAiProvider;
use super::providers::openrouter::OpenRouterProvider;
use super::types::{AiChatRequest, AiChatResponse, AiProvider, AiProviderConfig, AiProviderError};
use std::sync::Arc;

pub struct AiService {
    providers: Vec<(AiProviderConfig, Arc<dyn AiProviderAdapter>)>,
}

#[allow(dead_code)]
impl AiService {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    pub fn register_provider(&mut self, config: AiProviderConfig, api_key: Option<String>) {
        let adapter: Arc<dyn AiProviderAdapter> = match &config.provider {
            AiProvider::OpenAi => Arc::new(OpenAiProvider::new(
                api_key.unwrap_or_default(),
                config.base_url.clone(),
            )),
            AiProvider::Anthropic => Arc::new(AnthropicProvider::new(
                api_key.unwrap_or_default(),
                config.base_url.clone(),
            )),
            AiProvider::Gemini => Arc::new(GeminiProvider::new(
                api_key.unwrap_or_default(),
                config.base_url.clone(),
            )),
            AiProvider::OpenRouter => Arc::new(OpenRouterProvider::new(
                api_key.unwrap_or_default(),
                config.base_url.clone(),
            )),
            AiProvider::Ollama => Arc::new(OllamaProvider::new(config.base_url.clone())),
            AiProvider::Custom => Arc::new(OpenAiProvider::new(
                api_key.unwrap_or_default(),
                config.base_url.clone(),
            )),
        };
        self.providers.push((config, adapter));
    }

    pub fn get_provider(
        &self,
        provider: &AiProvider,
        model: &str,
    ) -> Option<(&AiProviderConfig, &dyn AiProviderAdapter)> {
        self.providers
            .iter()
            .find(|(c, _)| c.provider == *provider && (model.is_empty() || c.model == model))
            .map(|(c, a)| (c, a.as_ref()))
    }

    pub fn get_default_provider(&self) -> Option<(&AiProviderConfig, &dyn AiProviderAdapter)> {
        self.providers
            .iter()
            .find(|(c, _)| c.is_default)
            .or(self.providers.first())
            .map(|(c, a)| (c, a.as_ref()))
    }

    pub fn list_providers(&self) -> Vec<&AiProviderConfig> {
        self.providers.iter().map(|(c, _)| c).collect()
    }

    pub fn get_available_models(&self, provider: &AiProvider) -> Vec<String> {
        for (config, adapter) in &self.providers {
            if config.provider == *provider {
                return adapter
                    .available_models()
                    .into_iter()
                    .map(String::from)
                    .collect();
            }
        }
        Vec::new()
    }

    pub async fn chat(
        &self,
        config: &AiProviderConfig,
        request: AiChatRequest,
    ) -> Result<AiChatResponse, AiProviderError> {
        let adapter = self
            .providers
            .iter()
            .find(|(c, _)| c.provider == config.provider && c.model == config.model)
            .map(|(_, a)| a.as_ref())
            .ok_or_else(|| AiProviderError {
                message: format!(
                    "Provider '{}' with model '{}' not found",
                    config.provider, config.model
                ),
                code: None,
            })?;

        adapter.chat(request).await
    }

    pub async fn chat_stream(
        &self,
        config: &AiProviderConfig,
        request: AiChatRequest,
    ) -> Result<tokio::sync::mpsc::Receiver<String>, AiProviderError> {
        let adapter = self
            .providers
            .iter()
            .find(|(c, _)| c.provider == config.provider && c.model == config.model)
            .map(|(_, a)| a.as_ref())
            .ok_or_else(|| AiProviderError {
                message: format!(
                    "Provider '{}' with model '{}' not found",
                    config.provider, config.model
                ),
                code: None,
            })?;

        adapter.chat_stream(request).await
    }
}
