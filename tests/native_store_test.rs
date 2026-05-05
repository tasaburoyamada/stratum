use stratum::vector_stores::native::NativeVectorStore;
use stratum::vector_stores::base::VectorStore;
use stratum::vector_stores::types::VectorStoreQuery;
use stratum::core::schema::Node;
use half::f16;
use tempfile::tempdir;

#[tokio::test]
async fn test_native_vector_store_persistence() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let db_path = dir.path().join("test.redb");
    let db_path_str = db_path.to_str().unwrap();
    let dim = 3;

    {
        let store = NativeVectorStore::new(db_path_str, dim)?;
        
        let mut node1 = Node::new_text("Hello world".to_string());
        node1.embedding = Some(vec![f16::from_f32(1.0), f16::from_f32(0.0), f16::from_f32(0.0)]);
        
        let mut node2 = Node::new_text("Goodbye world".to_string());
        node2.embedding = Some(vec![f16::from_f32(0.0), f16::from_f32(1.0), f16::from_f32(0.0)]);

        store.add(vec![node1, node2]).await?;

        let query = VectorStoreQuery {
            query_embedding: Some(vec![f16::from_f32(1.0), f16::from_f32(0.0), f16::from_f32(0.0)]),
            similarity_top_k: 1,
            ..Default::default()
        };

        let result = store.query(query).await?;
        assert_eq!(result.ids.unwrap().len(), 1);
    }

    // Reopen and check persistence
    {
        let store = NativeVectorStore::new(db_path_str, dim)?;
        let query = VectorStoreQuery {
            query_embedding: Some(vec![f16::from_f32(0.0), f16::from_f32(1.0), f16::from_f32(0.0)]),
            similarity_top_k: 1,
            ..Default::default()
        };

        let result = store.query(query).await?;
        assert_eq!(result.ids.unwrap().len(), 1);
        // Should find node2
    }

    // Test filtering
    {
        let store = NativeVectorStore::new(db_path_str, dim)?;
        
        use stratum::vector_stores::types::{MetadataFilters, MetadataFilter, FilterOperator, FilterCondition};
        
        let mut node3 = Node::new_text("Special content".to_string());
        node3.embedding = Some(vec![f16::from_f32(0.5), f16::from_f32(0.5), f16::from_f32(0.0)]);
        node3.metadata.genre = Some("science".to_string());
        store.add(vec![node3]).await?;

        let query = VectorStoreQuery {
            query_embedding: Some(vec![f16::from_f32(0.5), f16::from_f32(0.5), f16::from_f32(0.0)]),
            similarity_top_k: 10,
            filters: Some(MetadataFilters {
                filters: vec![MetadataFilter {
                    key: "genre".to_string(),
                    value: serde_json::Value::String("science".to_string()),
                    operator: FilterOperator::Eq,
                }],
                condition: FilterCondition::And,
            }),
            ..Default::default()
        };

        let result = store.query(query).await?;
        assert_eq!(result.ids.as_ref().unwrap().len(), 1);
        
        // Query for non-existent genre
        let query_fail = VectorStoreQuery {
            query_embedding: Some(vec![f16::from_f32(0.5), f16::from_f32(0.5), f16::from_f32(0.0)]),
            similarity_top_k: 10,
            filters: Some(MetadataFilters {
                filters: vec![MetadataFilter {
                    key: "genre".to_string(),
                    value: serde_json::Value::String("history".to_string()),
                    operator: FilterOperator::Eq,
                }],
                condition: FilterCondition::And,
            }),
            ..Default::default()
        };
        let result_fail = store.query(query_fail).await?;
        assert_eq!(result_fail.ids.unwrap().len(), 0);
    }

    Ok(())
}
