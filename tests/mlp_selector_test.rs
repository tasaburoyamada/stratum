use stratum::indices::hierarchical::mlp_selector::MlpNodeSelector;
use stratum::indices::hierarchical::selector::NodeSelector;
use stratum::core::schema::Node;
use std::sync::Arc;

#[tokio::test]
async fn test_mlp_selector_integration() -> anyhow::Result<()> {
    let selector = MlpNodeSelector::new()?;
    let parent = Node::new_text("Context".to_string());
    let children = vec![
        Node::new_text("Choice 0".to_string()),
        Node::new_text("Choice 1".to_string()),
    ];
    
    let selected = selector.select("Query", &parent, &children).await?;
    assert!(!selected.is_empty());
    Ok(())
}
