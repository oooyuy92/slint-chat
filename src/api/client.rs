use anyhow::Result;
use reqwest::Client;
use serde::Serialize;

use crate::models::message::{Message, Role};

#[derive(Serialize)]
struct ApiMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ApiMessage>,
    stream: bool,
}

pub struct ApiClient {
    pub base_url: String,
    pub api_key: String,
    client: Client,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            client: Client::new(),
        }
    }

    /// 发起流式聊天请求，返回 reqwest::Response 供调用方用 parse_sse_stream 处理
    pub async fn chat_stream(
        &self,
        model: &str,
        messages: &[Message],
    ) -> Result<reqwest::Response> {
        let api_messages: Vec<ApiMessage> = messages
            .iter()
            .map(|m| ApiMessage {
                role: match m.role {
                    Role::User => "user".into(),
                    Role::Assistant => "assistant".into(),
                    Role::System => "system".into(),
                },
                content: m.content.clone(),
            })
            .collect();

        let body = ChatRequest {
            model: model.to_string(),
            messages: api_messages,
            stream: true,
        };

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?;

        Ok(response)
    }
}
