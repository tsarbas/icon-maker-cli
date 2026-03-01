use std::time::Duration;

use anyhow::{Context, bail};
use base64::Engine;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

pub struct OpenAiClient {
    api_key: String,
    client: reqwest::Client,
}

pub struct GenerationRequest {
    pub model: String,
    pub prompt: String,
    pub seed: Option<u64>,
    pub size: String,
}

#[derive(Debug, Serialize)]
struct OpenAiImageRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    size: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<&'a str>,
    n: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OpenAiImageResponse {
    data: Vec<ImageData>,
}

#[derive(Debug, Deserialize)]
struct ImageData {
    b64_json: Option<String>,
}

impl OpenAiClient {
    pub fn new(api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("valid reqwest client");
        Self { api_key, client }
    }

    pub async fn generate_master_icon(&self, req: &GenerationRequest) -> anyhow::Result<Vec<u8>> {
        let payload = build_image_request(req);

        let mut attempt = 0u32;
        loop {
            attempt += 1;

            let resp = self
                .client
                .post("https://api.openai.com/v1/images/generations")
                .bearer_auth(&self.api_key)
                .json(&payload)
                .send()
                .await
                .context("failed to call OpenAI image API")?;

            if resp.status().is_success() {
                let parsed: OpenAiImageResponse = resp
                    .json()
                    .await
                    .context("failed to parse OpenAI image response JSON")?;
                let b64 = parsed
                    .data
                    .first()
                    .and_then(|d| d.b64_json.as_ref())
                    .context("OpenAI image response did not contain b64_json data")?;

                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(b64)
                    .context("failed to decode base64 image payload")?;
                return Ok(bytes);
            }

            if should_retry(resp.status()) && attempt < 3 {
                sleep(Duration::from_millis(500 * attempt as u64)).await;
                continue;
            }

            let status = resp.status();
            let body = resp
                .text()
                .await
                .unwrap_or_else(|_| "<unable to read response body>".to_string());
            bail!("OpenAI image API error {}: {}", status.as_u16(), body);
        }
    }
}

fn should_retry(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

fn build_image_request(req: &GenerationRequest) -> OpenAiImageRequest<'_> {
    OpenAiImageRequest {
        model: &req.model,
        prompt: &req.prompt,
        size: &req.size,
        response_format: if req.model.starts_with("gpt-image-") {
            None
        } else {
            Some("b64_json")
        },
        n: 1,
        seed: req.seed,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{GenerationRequest, build_image_request};

    #[test]
    fn gpt_image_model_omits_response_format() {
        let req = GenerationRequest {
            model: "gpt-image-1.5".to_string(),
            prompt: "test".to_string(),
            seed: Some(42),
            size: "1024x1024".to_string(),
        };
        let payload = build_image_request(&req);
        let value: Value = serde_json::to_value(payload).expect("serialize payload");
        assert!(value.get("response_format").is_none());
    }

    #[test]
    fn non_gpt_image_model_includes_response_format() {
        let req = GenerationRequest {
            model: "dall-e-3".to_string(),
            prompt: "test".to_string(),
            seed: None,
            size: "1024x1024".to_string(),
        };
        let payload = build_image_request(&req);
        let value: Value = serde_json::to_value(payload).expect("serialize payload");
        assert_eq!(value["response_format"], "b64_json");
    }
}
