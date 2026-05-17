use stratum::core::schema::Node;
use stratum::postprocessors::base::NodePostprocessor;
use std::collections::HashMap;

#[test]
fn test_hash_determinism_vulnerability() {
    // Phase 1: Create two identical nodes with different HashMap insertion orders
    let mut meta1 = HashMap::new();
    meta1.insert("a".to_string(), serde_json::json!(1));
    meta1.insert("b".to_string(), serde_json::json!(2));

    let mut meta2 = HashMap::new();
    meta2.insert("b".to_string(), serde_json::json!(2));
    meta2.insert("a".to_string(), serde_json::json!(1));

    let mut node1 = Node::new_text("test".to_string());
    node1.metadata.extra = meta1;

    let mut node2 = Node::new_text("test".to_string());
    node2.metadata.extra = meta2;

    // Phase 2: Assert hashes are equal
    let hash1 = node1.hash();
    let hash2 = node2.hash();

    println!("Hash 1: {}", hash1);
    println!("Hash 2: {}", hash2);

    assert_eq!(hash1, hash2, "Node hash must be deterministic regardless of metadata insertion order");
}

#[tokio::test]
async fn test_purged_node_boost_recovery() {
    use stratum::postprocessors::vlog_bias::VlogBiasPostprocessor;
    use stratum::core::query_bundle::QueryBundle;
    use stratum::core::schema::NodeWithScore;

    // 1. Create a node with "Rust" keyword and metadata tags
    let mut node = Node::new_text("This is about Rust.".to_string());
    node.metadata.extra.insert("tags".to_string(), serde_json::json!(["Rust"]));
    
    // 2. Purge it (simulation of memory optimization)
    node.purge_text();

    // 3. Postprocess with a bias for "Rust"
    let postprocessor = VlogBiasPostprocessor::from_vlog("[[Rust]]", None);
    let nodes = vec![NodeWithScore { node, score: Some(1.0) }];

    let result = postprocessor.postprocess_nodes(nodes, &QueryBundle::new("query".to_string())).await.unwrap();

    // 4. Check if boosted. Should SUCCEED now because we check metadata tags as fallback.
    assert!(result[0].score.unwrap() > 1.0, "Purged node should be boosted via metadata tags fallback");
}
