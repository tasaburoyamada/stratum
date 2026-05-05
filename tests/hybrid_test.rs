use stratum::retrievers::hybrid::{HybridRetriever, HybridMode};
use stratum::retrievers::base::Retriever;
use stratum::core::schema::{Node, NodeWithScore};
use stratum::core::query_bundle::QueryBundle;
use std::sync::Arc;
use async_trait::async_trait;

struct MockRetriever {
    results: Vec<NodeWithScore>,
}

#[async_trait]
impl Retriever for MockRetriever {
    async fn retrieve(&self, _query: QueryBundle) -> anyhow::Result<Vec<NodeWithScore>> {
        Ok(self.results.clone())
    }
}

#[tokio::test]
async fn test_hybrid_retriever_weighted_sum() -> anyhow::Result<()> {
    let node1 = Node::new_text("Node 1".to_string());
    let node2 = Node::new_text("Node 2".to_string());
    
    let r1 = Arc::new(MockRetriever {
        results: vec![
            NodeWithScore { node: node1.clone(), score: Some(0.8) },
            NodeWithScore { node: node2.clone(), score: Some(0.2) },
        ]
    });
    
    let r2 = Arc::new(MockRetriever {
        results: vec![
            NodeWithScore { node: node1.clone(), score: Some(0.1) },
            NodeWithScore { node: node2.clone(), score: Some(0.9) },
        ]
    });

    // Equal weights
    let hybrid = HybridRetriever::new(vec![r1, r2], HybridMode::WeightedSum, None);
    let results = hybrid.retrieve(QueryBundle::new("test".to_string())).await?;

    assert_eq!(results.len(), 2);
    // Node 2 total = 0.2 + 0.9 = 1.1
    // Node 1 total = 0.8 + 0.1 = 0.9
    assert_eq!(results[0].node.id_, node2.id_);
    assert!((results[0].score.unwrap() - 1.1).abs() < 1e-6);

    Ok(())
}

#[tokio::test]
async fn test_hybrid_retriever_rrf() -> anyhow::Result<()> {
    let node1 = Node::new_text("Node 1".to_string());
    let node2 = Node::new_text("Node 2".to_string());
    
    // r1: [Node1, Node2]
    // r2: [Node2, Node1]
    let r1 = Arc::new(MockRetriever {
        results: vec![
            NodeWithScore { node: node1.clone(), score: Some(1.0) },
            NodeWithScore { node: node2.clone(), score: Some(0.5) },
        ]
    });
    let r2 = Arc::new(MockRetriever {
        results: vec![
            NodeWithScore { node: node2.clone(), score: Some(1.0) },
            NodeWithScore { node: node1.clone(), score: Some(0.5) },
        ]
    });

    let hybrid = HybridRetriever::new(vec![r1, r2], HybridMode::ReciprocalRankFusion, None);
    let results = hybrid.retrieve(QueryBundle::new("test".to_string())).await?;

    assert_eq!(results.len(), 2);
    // Both should have same score since their ranks are swapped
    assert!((results[0].score.unwrap() - results[1].score.unwrap()).abs() < 1e-6);

    Ok(())
}
