use futures::stream::BoxStream;
use stratum::indices::hierarchical::HierarchicalIndex;
use stratum::retrievers::hierarchical_retriever::HierarchicalRetriever;
use stratum::retrievers::base::Retriever;
use stratum::core::schema::Node;
use stratum::storage::storage_context::StorageContext;
use stratum::llm::LlmClient;
use stratum::core::query_bundle::QueryBundle;
use stratum::embeddings::base::Embedding;
use stratum::indices::hierarchical::selector::NodeSelector;
use std::sync::Arc;
use async_trait::async_trait;
use half::f16;
use log::{info, error};

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

    fn stream_complete(&self, prompt: &str) -> BoxStream<'static, anyhow::Result<String>> { let prompt = prompt.to_string(); let client = self.clone_box(); use futures::stream::{self, StreamExt}; stream::once(async move { client.complete(&prompt).await }).boxed() }
    fn clone_box(&self) -> Box<dyn LlmClient> {
        Box::new(Self)
    }
}

#[derive(Debug)]
struct DummyEmbedding;

#[async_trait]
impl Embedding for DummyEmbedding {
    async fn get_text_embedding(&self, text: &str) -> anyhow::Result<Vec<f16>> {
        Ok(vec![f16::from_f32(text.len() as f32); 384])
    }
    async fn get_text_embedding_batch(&self, _texts: Vec<String>, _batch_size: usize) -> anyhow::Result<Vec<Vec<f16>>> { 
        unimplemented!("DummyEmbedding does not implement batch embedding for tests")
    }
    fn model_name(&self) -> &str { "dummy" }
}

// Mock NodeSelector to always return selected indices for persistence test
struct MockNodeSelector;

#[async_trait]
impl NodeSelector for MockNodeSelector {
    async fn select(
        &self,
        _query_str: &str,
        _parent_node: &Node,
        children_nodes: &[Node],
    ) -> anyhow::Result<Vec<usize>> {
        if children_nodes.is_empty() {
            Ok(vec![])
        } else {
            // Select all children for simplicity in this persistence test
            Ok((0..children_nodes.len()).collect())
        }
    }
}

#[tokio::test]
async fn test_hierarchical_index_end_to_end() -> anyhow::Result<()> {
    let storage_context = StorageContext::from_defaults();
    let llm = Arc::new(MockHierarchicalLlm);
    let embed_model = Arc::new(DummyEmbedding);

    let nodes = vec![
        Node::new_text("The capital of France is Paris.".to_string()),
        Node::new_text("The capital of Germany is Berlin.".to_string()),
        Node::new_text("The capital of Japan is Tokyo.".to_string()),
    ];

    // 1. Build Index
    let index = Arc::new(HierarchicalIndex::from_nodes(
        nodes,
        storage_context,
        llm.clone(),
        embed_model.clone(),
    ).await?);

    assert!(!index.root_node_ids.is_empty());

    // 2. Setup Retriever
    let retriever = HierarchicalRetriever::new(index, 2)?;

    // 3. Retrieve
    let query_bundle = QueryBundle::new("What is the capital of France?".to_string());
    let results = retriever.retrieve(query_bundle).await?;

    assert!(!results.is_empty());
    // Since our mock always picks choice 0, it should find "Paris" (if chunks were small enough)
    let content = results[0].node.get_content(None)?;
    assert!(content.contains("summary") || content.contains("France"));

    Ok(())
}

#[tokio::test]
async fn test_hierarchical_index_persistence() -> anyhow::Result<()> {
    env_logger::init(); // Initialize logger
    use tempfile::tempdir;
    let dir = tempdir()?;
    let persist_dir = dir.path().to_str().unwrap();
    let llm = Arc::new(MockHierarchicalLlm);
    let embed_model = Arc::new(DummyEmbedding);
    let mock_selector = Arc::new(MockNodeSelector); // Instantiate mock selector

    let index_id: String;

    {
        let storage_context = StorageContext::from_dir(persist_dir)?;
        let nodes = vec![
            Node::new_text("Persistent context A".to_string()),
            Node::new_text("Persistent context B".to_string()),
        ];
        let index = HierarchicalIndex::from_nodes(nodes, storage_context.clone(), llm.clone(), embed_model.clone()).await?;
        log::info!("Original index root_node_ids: {:?}", index.root_node_ids);
        index_id = index.index_id.clone();
        storage_context.persist(persist_dir).await?;
    }

    // Re-load
    {
        let storage_context = StorageContext::from_dir(persist_dir)?;
        let index = HierarchicalIndex::from_storage_context_async(storage_context, llm.clone(), embed_model.clone(), index_id).await?;
        log::info!("Reloaded index root_node_ids: {:?}", index.root_node_ids);
        assert!(!index.root_node_ids.is_empty());
        
        let retriever = HierarchicalRetriever::with_selector(Arc::new(index), mock_selector.clone(), 1);
        let results = retriever.retrieve(QueryBundle::new("Persistent query".to_string())).await?;
        assert!(!results.is_empty());
    }

    Ok(())
}.is_empty());
    }

    Ok(())
}