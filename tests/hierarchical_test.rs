use stratum::indices::hierarchical::HierarchicalIndex;
use stratum::retrievers::hierarchical_retriever::HierarchicalRetriever;
use stratum::retrievers::base::Retriever;
use stratum::core::schema::Node;
use stratum::storage::storage_context::StorageContext;
use stratum::llm::LlmClient;
use stratum::core::query_bundle::QueryBundle;
use std::sync::Arc;
use async_trait::async_trait;

struct MockHierarchicalLlm;

#[async_trait]
impl LlmClient for MockHierarchicalLlm {
    async fn complete(&self, prompt: &str) -> anyhow::Result<String> {
        if prompt.contains("Summarize") {
            Ok("This is a summary of the provided chunks.".to_string())
        } else if prompt.contains("Choice") {
            // Mock selection: always select the first choice
            Ok("0".to_string())
        } else {
            Ok("Mock response".to_string())
        }
    }
}

#[tokio::test]
async fn test_hierarchical_index_end_to_end() -> anyhow::Result<()> {
    let storage_context = StorageContext::from_defaults();
    let llm = Arc::new(MockHierarchicalLlm);

    let nodes = vec![
        Node::new_text("The capital of France is Paris.".to_string()),
        Node::new_text("The capital of Germany is Berlin.".to_string()),
        Node::new_text("The capital of Japan is Tokyo.".to_string()),
    ];

    // 1. Build Index
    let index = Arc::new(HierarchicalIndex::from_nodes(
        nodes,
        storage_context,
        llm.clone()
    ).await?);

    assert!(!index.root_node_ids.is_empty());

    // 2. Setup Retriever
    let retriever = HierarchicalRetriever::new(index, 2);

    // 3. Retrieve
    let query_bundle = QueryBundle::new("What is the capital of France?".to_string());
    let results = retriever.retrieve(query_bundle).await?;

    assert!(!results.is_empty());
    // Since our mock always picks choice 0, it should find "Paris" (if chunks were small enough)
    let content = results[0].node.get_content(None)?;
    assert!(content.contains("summary") || content.contains("France"));

    Ok(())
}
