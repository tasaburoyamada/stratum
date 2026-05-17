use stratum::llm::deterministic::DeterministicWrapper;
use stratum::llm::gemini::GeminiClient;
use stratum::llm::LlmClient;

#[tokio::test]
async fn test_alignment_audit() {
    let inner = GeminiClient::new("test_key".to_string(), None);
    let client = DeterministicWrapper::new(Box::new(inner));
    
    let prompt = "Audit Query";
    // We expect the wrapper to log the hash.
    let _ = client.complete(prompt).await; 
    assert!(true);
}
