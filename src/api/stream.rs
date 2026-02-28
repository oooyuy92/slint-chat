use anyhow::Result;
use futures_util::StreamExt;
use reqwest::Response;
use serde::Deserialize;

#[derive(Deserialize)]
struct SseDelta {
    content: Option<String>,
    reasoning_content: Option<String>,
}

#[derive(Deserialize)]
struct SseChoice {
    delta: SseDelta,
}

#[derive(Deserialize)]
struct SseChunk {
    choices: Vec<SseChoice>,
}

/// Token type distinguishing content from reasoning
pub enum StreamToken {
    Content(String),
    Reasoning(String),
}

/// 解析 OpenAI SSE 流，每收到一个 token 调用 on_token 回调
pub async fn parse_sse_stream<F>(response: Response, mut on_token: F) -> Result<()>
where
    F: FnMut(StreamToken),
{
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);

        for line in text.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                if data.trim() == "[DONE]" {
                    return Ok(());
                }
                if let Ok(parsed) = serde_json::from_str::<SseChunk>(data) {
                    for choice in parsed.choices {
                        if let Some(reasoning) = choice.delta.reasoning_content {
                            if !reasoning.is_empty() {
                                on_token(StreamToken::Reasoning(reasoning));
                            }
                        }
                        if let Some(content) = choice.delta.content {
                            if !content.is_empty() {
                                on_token(StreamToken::Content(content));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
