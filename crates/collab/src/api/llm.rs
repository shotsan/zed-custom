use crate::{AppState, Error, Result};
use axum::{
    body::StreamBody,
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use cloud_llm_client::{
    CompletionBody, LanguageModel, LanguageModelId, LanguageModelProvider, ListModelsResponse,
    SERVER_SUPPORTS_STATUS_MESSAGES_HEADER_NAME,
};
use reqwest::Client;
use std::sync::{Arc, OnceLock};

fn http_client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(Client::new)
}

pub async fn get_models(
    Extension(_app): Extension<Arc<AppState>>,
) -> Result<Json<ListModelsResponse>, Error> {
    Ok(Json(ListModelsResponse {
        models: vec![
            LanguageModel {
                provider: LanguageModelProvider::OpenAi,
                id: LanguageModelId("gpt-4o".into()),
                display_name: "GPT-4o".to_string(),
                max_token_count: 128000,
                max_token_count_in_max_mode: None,
                max_output_tokens: 4096,
                supports_tools: true,
                supports_images: true,
                supports_thinking: false,
                supports_streaming_tools: false,
                supports_parallel_tool_calls: true,
            },
            LanguageModel {
                provider: LanguageModelProvider::Anthropic,
                id: LanguageModelId("claude-3-5-sonnet-20240620".into()),
                display_name: "Claude 3.5 Sonnet".to_string(),
                max_token_count: 200000,
                max_token_count_in_max_mode: None,
                max_output_tokens: 8192,
                supports_tools: true,
                supports_images: true,
                supports_thinking: false,
                supports_streaming_tools: false,
                supports_parallel_tool_calls: false,
            },
        ],
        default_model: Some(LanguageModelId("gpt-4o".into())),
        default_fast_model: Some(LanguageModelId("gpt-4o".into())),
        recommended_models: vec![
            LanguageModelId("gpt-4o".into()),
            LanguageModelId("claude-3-5-sonnet-20240620".into()),
        ],
    }))
}

pub async fn chat_completions(
    Extension(app): Extension<Arc<AppState>>,
    Json(payload): Json<CompletionBody>,
) -> Result<impl IntoResponse, Error> {
    let client = http_client();

    let req = match payload.provider {
        LanguageModelProvider::OpenAi => {
            let key = app
                .config
                .openai_api_key
                .as_ref()
                .map(|s| s.as_ref())
                .unwrap_or_default();
            client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", key))
                .header("Content-Type", "application/json")
                .json(&payload.provider_request)
        }
        LanguageModelProvider::Anthropic => {
            let key = app
                .config
                .anthropic_api_key
                .as_ref()
                .map(|s| s.as_ref())
                .unwrap_or_default();
            client
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&payload.provider_request)
        }
        _ => {
            return Err(Error::http(
                StatusCode::BAD_REQUEST,
                format!("Unsupported provider: {:?}", payload.provider),
            ));
        }
    };

    let res = req.send().await.map_err(|e| {
        log::error!("Failed to reach LLM provider: {}", e);
        Error::http(
            StatusCode::BAD_GATEWAY,
            "Failed to reach LLM provider".to_string(),
        )
    })?;

    let status = res.status();
    if !status.is_success() {
        let body = res.text().await.unwrap_or_default();
        log::error!("LLM Provider returned {}: {}", status, body);
        return Err(Error::http(
            status,
            format!("LLM Provider returned an error: {}", body),
        ));
    }

    let stream = res.bytes_stream();
    let body = StreamBody::new(stream);

    Ok((
        StatusCode::OK,
        [
            (axum::http::header::CONTENT_TYPE, "text/event-stream"),
            (
                axum::http::HeaderName::from_static(SERVER_SUPPORTS_STATUS_MESSAGES_HEADER_NAME),
                "true",
            ),
        ],
        body,
    ))
}
