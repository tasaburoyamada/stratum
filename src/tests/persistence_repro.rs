use stratum::vector_stores::native::NativeVectorStore;
use stratum::vector_stores::base::VectorStore;
use stratum::core::schema::Node;
use half::f16;
use tempfile::tempdir;

#[tokio::test]
async fn test_native_vector_store_persistence_bug() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("test_db.redb");
    let db_path_str = db_path.to_str().unwrap();

    let dim = 384;
    
    // 1. First run: Create and add a node
    {
        let store = NativeVectorStore::new(db_path_str, dim).unwrap();
        let mut node = Node::new_text("Hello persistent world".to_string());
        node.embedding = Some(vec![f16::from_f32(1.0); dim]);
        store.add(vec![node]).await.unwrap();
    }

    // 2. Second run: Re-open the store
    {
        let store = NativeVectorStore::new(db_path_str, dim).unwrap();
        // If the bug exists, rebuild_index or subsequent queries might fail or show 0 nodes
        // but the core issue is Database::create overwriting.
        
        // Let's check if the node still exists via a query
        let query = stratum::vector_stores::types::VectorStoreQuery {
            query_embedding: Some(vec![f16::from_f32(1.0); dim]),
            similarity_top_k: 1,
            ..Default::default()
        };
        
        let result = store.query(query).await.unwrap();
        assert_eq!(result.ids.unwrap().len(), 1, "Node should persist across restarts");
    }
}
