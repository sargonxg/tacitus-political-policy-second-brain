//! Fail closed before constructing providers; container egress isolation is a second boundary.
pub fn enabled() -> bool {
    std::env::var("TACITUS_LOCAL_ONLY").as_deref() == Ok("1")
}
pub fn host() -> String {
    std::env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://ollama:11434".into())
}
pub fn model(embedding: bool) -> String {
    std::env::var(if embedding {
        "EDGEQUAKE_EMBEDDING_MODEL"
    } else {
        "EDGEQUAKE_LLM_MODEL"
    })
    .unwrap_or_else(|_| {
        if embedding {
            "embeddinggemma:latest"
        } else {
            "qwen3:4b"
        }
        .into()
    })
}
pub fn validate_identity(provider: &str, requested: &str, embedding: bool) -> Result<(), String> {
    if !enabled() {
        return Ok(());
    }
    check_identity(provider, requested, &model(embedding), &host())
}
pub fn check_identity(
    provider: &str,
    requested: &str,
    allowed: &str,
    endpoint: &str,
) -> Result<(), String> {
    if provider != "ollama"
        || requested != allowed
        || requested.to_lowercase().contains("cloud")
        || ![
            "http://ollama:11434",
            "http://127.0.0.1:11434",
            "http://host.docker.internal:11434",
        ]
        .contains(&endpoint)
    {
        return Err("Tacitus local-only policy rejects unapproved provider/model/endpoint; no fallback allowed".into());
    }
    Ok(())
}
pub fn startup() -> Result<(), String> {
    if !enabled() {
        return Ok(());
    }
    for key in [
        "OPENAI_API_KEY",
        "ANTHROPIC_API_KEY",
        "GEMINI_API_KEY",
        "MISTRAL_API_KEY",
        "AZURE_OPENAI_API_KEY",
        "OLLAMA_API_KEY",
        "OPENAI_COMPATIBLE_API_KEY",
        "EDGEQUAKE_CHAT_API_KEY",
        "EDGEQUAKE_VISION_API_KEY",
        "EDGEQUAKE_DECISION_API_KEY",
        "EDGEQUAKE_EMBEDDING_API_KEY",
        "GOOGLE_APPLICATION_CREDENTIALS",
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
        "EDGEQUAKE_CHAT_BASE_URL",
        "EDGEQUAKE_EMBEDDING_BASE_URL",
        "OLLAMA_EMBEDDING_HOST",
    ] {
        if std::env::var(key).is_ok_and(|v| !v.trim().is_empty()) {
            return Err(format!("Remove {key} from the Tacitus server environment; no provider credentials are inherited"));
        }
    }
    validate_identity(
        &std::env::var("EDGEQUAKE_LLM_PROVIDER").unwrap_or_default(),
        &model(false),
        false,
    )?;
    validate_identity(
        &std::env::var("EDGEQUAKE_EMBEDDING_PROVIDER").unwrap_or_default(),
        &model(true),
        true,
    )?;
    if std::env::var("OLLAMA_NO_CLOUD").as_deref() != Ok("1") {
        return Err(
            "OLLAMA_NO_CLOUD=1 must be configured; also verify it on the actual Ollama server"
                .into(),
        );
    }
    if std::env::var("EDGEQUAKE_ALLOW_MOCK_PROVIDER").as_deref() == Ok("1")
        || std::env::var("EDGEQUAKE_ALLOW_TEST_PROVIDER_OVERRIDE").as_deref() == Ok("1")
    {
        return Err("Mock/test provider overrides are forbidden in Tacitus runtime".into());
    }
    Ok(())
}
pub fn reject_connection() -> Result<(), String> {
    if enabled() {
        Err("Saved provider connections are disabled by the Tacitus local-only policy".into())
    } else {
        Ok(())
    }
}
