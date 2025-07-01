use async_trait::async_trait;
use async_openai::{
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
    Client,
};
use std::error::Error;

// Extensible AI backend trait
#[async_trait]
pub trait AiBackend: Send + Sync {
    async fn chat(&self, message: &str) -> Result<String, Box<dyn Error + Send + Sync>>;
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
}

// OpenAI ChatGPT implementation using async-openai SDK
pub struct OpenAiBackend {
    client: Client<async_openai::config::OpenAIConfig>,
}

impl OpenAiBackend {
    pub fn new(api_key: String) -> Self {
        let client = Client::with_config(async_openai::config::OpenAIConfig::new().with_api_key(api_key));
        Self { client }
    }
}

#[async_trait]
impl AiBackend for OpenAiBackend {
    async fn chat(&self, message: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o")
            .max_tokens(500u32)
            .messages(vec![
                ChatCompletionRequestUserMessageArgs::default()
                    .content(message)
                    .build()?
                    .into()
            ])
            .build()?;

        let response = self.client.chat().create(request).await?;

        if let Some(choice) = response.choices.first() {
            if let Some(content) = &choice.message.content {
                Ok(content.trim().to_string())
            } else {
                Err("No content in OpenAI response".into())
            }
        } else {
            Err("No response from OpenAI".into())
        }
    }

    fn name(&self) -> &'static str {
        "OpenAI ChatGPT"
    }
}

// AI Backend factory for future extensibility
pub fn create_ai_backend() -> Result<Box<dyn AiBackend>, Box<dyn Error + Send + Sync>> {
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        Ok(Box::new(OpenAiBackend::new(api_key)))
    } else {
        Err("OPENAI_API_KEY environment variable not set".into())
    }
}