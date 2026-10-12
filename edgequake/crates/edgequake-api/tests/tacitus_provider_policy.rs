use edgequake_api::tacitus_provider_policy::check_identity;
#[test]
fn exact_local_identity_rejects_cloud_model_endpoint_and_provider_overrides() {
    assert!(check_identity("ollama", "qwen3:4b", "qwen3:4b", "http://ollama:11434").is_ok());
    for (provider, model, host) in [
        ("openai", "qwen3:4b", "http://ollama:11434"),
        ("mock", "qwen3:4b", "http://ollama:11434"),
        ("ollama", "other", "http://ollama:11434"),
        ("ollama", "qwen3:4b", "https://unapproved.invalid"),
        (
            "ollama",
            "qwen3:4b",
            "http://ollama:11434@unapproved.invalid",
        ),
    ] {
        assert!(check_identity(provider, model, "qwen3:4b", host).is_err());
    }
    assert!(check_identity("ollama", "x-cloud", "x-cloud", "http://ollama:11434").is_err());
}
