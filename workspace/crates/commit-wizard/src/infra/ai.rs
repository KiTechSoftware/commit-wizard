//! Runs the configured AI backend (through `cliai`) and returns its text.

use cliai::{AiBackend, Copilot, GenerateRequest, Ollama};

use crate::engine::{Error, ErrorCode, models::policy::enforcement::AiProvider};

/// Model used for Ollama when `ai.model` is not set.
pub const DEFAULT_OLLAMA_MODEL: &str = "llama3.2";

pub fn generate(
    provider: AiProvider,
    model: Option<&str>,
    request: &GenerateRequest,
) -> Result<String, Error> {
    let backend: Box<dyn AiBackend> = match provider {
        AiProvider::Copilot => Box::new(Copilot::new()),
        AiProvider::Ollama => Box::new(Ollama::new(model.unwrap_or(DEFAULT_OLLAMA_MODEL))),
    };
    backend
        .generate(request)
        .map(|response| response.text)
        .map_err(|err| {
            ErrorCode::AiProviderFailed
                .error()
                .with_context("provider", provider.as_str().to_string())
                .with_context("error", err.to_string())
        })
}
