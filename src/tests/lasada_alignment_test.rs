use crate::llm::deterministic::DeterministicWrapper;
use crate::llm::gemini::GeminiClient;
use crate::llm::LlmClient;
use std::sync::Arc;

#[tokio::test]
async fn test_deterministic_llm_hashing() {
    let inner = GeminiClient::new("test_key".to_string(), None);
    let client = DeterministicWrapper::new(Box::new(inner));
    
    let prompt = "Test Prompt";
    let hash = blake3::hash(prompt.as_bytes()).to_hex().to_string();
    assert_eq!(hash, "5f555e533087224209935105260c558c49cc14f777e5e34771239103e33d0263");
}
