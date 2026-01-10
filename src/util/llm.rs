use crate::{config::ENV, error::AppError, model::llm::GeminiResponse};
use anyhow::Context;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::json;

pub async fn gemini(prompt: impl Into<String>, model: Option<&str>) -> Result<String, AppError> {
    let model = model.unwrap_or("gemini-2.0-flash-lite");

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        "x-goog-api-key",
        HeaderValue::from_str(&ENV.gemini_api_key)?,
    );

    let body = json!({
    "contents": [
        {
            "parts": [
                {
                    "text": prompt.into()
                }
            ]
        }
    ]
    });

    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            model,
        ))
        .headers(headers)
        .json(&body)
        .send()
        .await
        .context("failed to send request to Gemini API")?;
    if !response.status().is_success() {
        return Err(AppError::Other(anyhow::anyhow!(
            "Gemini API request failed with status: {}",
            response.status()
        )));
    }

    let value: GeminiResponse = response
        .json()
        .await
        .context("failed to parse Gemini API response")?;

    value
        .candidates
        .first()
        .and_then(|candidate| {
            candidate
                .content
                .parts
                .first()
                .map(|part| part.text.clone())
        })
        .ok_or_else(|| AppError::Other(anyhow::anyhow!("no content in Gemini API response")))
}
