use reqwest::Client;
use serde::Deserialize;

use crate::audio::{wav, TARGET_SAMPLE_RATE};

const STT_URL: &str = "https://api.sarvam.ai/speech-to-text";

#[derive(Debug, Deserialize)]
struct SpeechToTextResponse {
    transcript: String,
}

#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    error: ApiErrorDetails,
}

#[derive(Debug, Deserialize)]
struct ApiErrorDetails {
    message: String,
}

pub async fn transcribe_pcm(api_key: &str, pcm_bytes: &[u8], mode: &str) -> Result<String, String> {
    if pcm_bytes.is_empty() {
        return Err("No audio recorded".to_string());
    }

    let wav_bytes = wav::pcm_to_wav(pcm_bytes, TARGET_SAMPLE_RATE);

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;

    let file_part = reqwest::multipart::Part::bytes(wav_bytes)
        .file_name("recording.wav")
        .mime_str("audio/wav")
        .map_err(|e| e.to_string())?;

    let form = reqwest::multipart::Form::new()
        .part("file", file_part)
        .text("model", "saaras:v3")
        .text("mode", mode.to_string())
        .text("language_code", "unknown");

    let response = client
        .post(STT_URL)
        .header("api-subscription-key", api_key)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Speech-to-text request failed: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        if let Ok(parsed) = serde_json::from_str::<ApiErrorBody>(&body) {
            return Err(parsed.error.message);
        }
        return Err(format!("Speech-to-text API error {status}: {body}"));
    }

    let parsed: SpeechToTextResponse = response.json().await.map_err(|e| e.to_string())?;
    Ok(parsed.transcript.trim().to_string())
}
