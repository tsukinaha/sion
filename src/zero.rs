use std::sync::Arc;

use reqwest::Client;
use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    config::ClientConfig,
    models::Model,
};

#[derive(Debug, Clone)]
pub struct SionClient {
    client: Client,
    config: Arc<ClientConfig>,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    messages: Vec<ChatMessage>,
    model: String,
}

impl SionClient {
    pub fn new(config: ClientConfig) -> Self {
        let client = Client::builder()
            .https_only(true)
            .http2_adaptive_window(true)
            .build()
            .expect("failed to build http client");

        Self {
            client,
            config: Arc::new(config),
        }
    }

    pub async fn request_new_hint(&self, content: String, model: Model) -> anyhow::Result<String> {
        let request_body = ChatCompletionRequest {
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content,
            }],
            model: model.to_string(),
        };

        let response = self
            .client
            .post(self.config.base_url.clone())
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.config.token))
            .json(&request_body)
            .send()
            .await?;

        let response_body: ChatCompletionResponse = response.json().await?;
        if let Some(choice) = response_body.choices.first() {
            Ok(choice.message.content.clone())
        } else {
            Err(anyhow::anyhow!("No response received"))
        }
    }
}

#[derive(Deserialize)]
pub struct ChatCompletionResponse {
    choices: Vec<ChatCompletionMessage>,
}

#[derive(Deserialize)]
pub struct ChatCompletionMessage {
    message: ChatMessage,
}
