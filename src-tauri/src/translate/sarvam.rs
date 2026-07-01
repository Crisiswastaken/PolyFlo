use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::stt::sarvam as stt;

/// Sarvam Translate REST API — `sarvam-translate:v1`
/// https://docs.sarvam.ai/api-reference-docs/getting-started/models/sarvam-translate
const TRANSLATE_URL: &str = "https://api.sarvam.ai/translate";
const LANGUAGE_ID_URL: &str = "https://api.sarvam.ai/text-lid";

#[derive(Serialize)]
struct TranslateRequest {
    input: String,
    source_language_code: String,
    target_language_code: String,
    model: String,
}

#[derive(Serialize)]
struct LanguageIdRequest {
    input: String,
}

#[derive(Deserialize)]
struct TranslateResponse {
    translated_text: String,
}

#[derive(Deserialize)]
struct LanguageIdResponse {
    language_code: Option<String>,
}

/// Transcribe audio and return English text.
/// Tries Saaras `translate` mode first, then transcribe + text translation.
pub async fn transcribe_to_english(api_key: &str, pcm_bytes: &[u8]) -> Result<String, String> {
    match stt::transcribe_pcm(api_key, pcm_bytes, "translate").await {
        Ok(text) if !text.trim().is_empty() => {
            tracing::info!("Speech-to-text translate mode succeeded");
            return Ok(text);
        }
        Ok(_) => tracing::warn!("Speech-to-text translate mode returned empty text"),
        Err(e) => tracing::warn!("Speech-to-text translate mode failed: {e}"),
    }

    let raw = stt::transcribe_pcm(api_key, pcm_bytes, "transcribe").await?;
    if raw.trim().is_empty() {
        return Ok(raw);
    }

    translate_to_english(api_key, &raw).await
}

pub async fn translate_to_english(api_key: &str, text: &str) -> Result<String, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(text.to_string());
    }

    let source = identify_language(api_key, trimmed).await?;
    if source == "en-IN" {
        return Ok(trimmed.to_string());
    }

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let request = TranslateRequest {
        input: trimmed.to_string(),
        source_language_code: source,
        target_language_code: "en-IN".to_string(),
        model: "sarvam-translate:v1".to_string(),
    };

    let response = client
        .post(TRANSLATE_URL)
        .header("api-subscription-key", api_key)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Translate request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Translate API error {status}: {body}"));
    }

    let parsed: TranslateResponse = response.json().await.map_err(|e| e.to_string())?;
    let translated = parsed.translated_text.trim().to_string();
    if translated.is_empty() {
        return Err("Translate API returned empty text".to_string());
    }
    Ok(translated)
}

async fn identify_language(api_key: &str, text: &str) -> Result<String, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .post(LANGUAGE_ID_URL)
        .header("api-subscription-key", api_key)
        .header("Content-Type", "application/json")
        .json(&LanguageIdRequest {
            input: text.to_string(),
        })
        .send()
        .await
        .map_err(|e| format!("Language detection request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Language detection API error {status}: {body}"));
    }

    let parsed: LanguageIdResponse = response.json().await.map_err(|e| e.to_string())?;
    parsed
        .language_code
        .filter(|code| !code.is_empty())
        .ok_or_else(|| "Could not detect source language for translation".to_string())
}
