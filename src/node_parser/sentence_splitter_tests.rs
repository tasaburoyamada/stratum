use crate::node_parser::sentence_splitter::SentenceSplitter;
use crate::core::schema::Node;
use crate::core::config::SplitterConfig;
use crate::core::ingestion::transformation::Transformation;

#[tokio::test]
async fn test_basic_split() {
    let config = SplitterConfig { chunk_size: 10, chunk_overlap: 2, ..Default::default() };
    let splitter = SentenceSplitter::new(config);
    
    let text = "This is a sentence. And this is another one.";
    let node = Node::new_text(text.to_string());
    
    let result = splitter.transform(vec![node]).await.unwrap();
    assert!(!result.is_empty());
}

#[tokio::test]
async fn test_metadata_overflow_skip() {
    let config = SplitterConfig { chunk_size: 5, chunk_overlap: 1, ..Default::default() };
    let splitter = SentenceSplitter::new(config);
    
    let mut node = Node::new_text("Small text.".to_string());
    node.metadata.file_path = Some("A very long metadata string that definitely exceeds five tokens in length. ".repeat(10));
    
    let result = splitter.transform(vec![node]).await.unwrap();
    assert_eq!(result.len(), 1, "Should skip split and return original node");
}
