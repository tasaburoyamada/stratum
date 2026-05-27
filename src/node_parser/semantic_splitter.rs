use crate::core::schema::Node;
use crate::core::ingestion::transformation::Transformation;
use crate::embeddings::base::Embedding;
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use std::sync::{Arc, OnceLock};

static SENTENCE_REGEX: OnceLock<Regex> = OnceLock::new();

pub struct SemanticSplitter {
    pub embed_model: Arc<dyn Embedding>,
    pub buffer_size: usize,
    pub breakpoint_percentile_threshold: f32,
    pub embedding_batch_size: usize,
}

impl std::fmt::Debug for SemanticSplitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SemanticSplitter")
            .field("buffer_size", &self.buffer_size)
            .field("breakpoint_percentile_threshold", &self.breakpoint_percentile_threshold)
            .field("embedding_batch_size", &self.embedding_batch_size)
            .finish()
    }
}

impl SemanticSplitter {
    pub fn new(embed_model: Arc<dyn Embedding>) -> Self {
        Self {
            embed_model,
            buffer_size: 1,
            breakpoint_percentile_threshold: 95.0,
            embedding_batch_size: 32,
        }
    }

    fn get_sentence_regex(&self) -> &Regex {
        SENTENCE_REGEX.get_or_init(|| {
            Regex::new(r#"(?m)[^.!?。！？]+[.!?。！？]?["'」』]?"#)
                .expect("Failed to initialize sentence regex")
        })
    }

    async fn get_sentence_embeddings(&self, sentences: &[String]) -> Result<Vec<Vec<f32>>> {
        let embeddings = self.embed_model.get_text_embedding_batch(sentences.to_vec(), self.embedding_batch_size).await?;
        Ok(embeddings.into_iter().map(|e| e.iter().map(|&x| f32::from(x)).collect()).collect())
    }

    fn calculate_distances(&self, embeddings: &[Vec<f32>]) -> Vec<f32> {
        let mut distances = Vec::new();
        if embeddings.len() < 2 {
            return distances;
        }

        for i in 0..embeddings.len() - 1 {
            let left_start = i.saturating_sub(self.buffer_size - 1);
            let left_end = i;
            let right_start = i + 1;
            let right_end = (i + self.buffer_size).min(embeddings.len() - 1);

            let left_avg = self.average_embeddings(&embeddings[left_start..=left_end]);
            let right_avg = self.average_embeddings(&embeddings[right_start..=right_end]);

            let dist = 1.0 - cosine_similarity_f32(&left_avg, &right_avg);
            distances.push(dist);
        }
        distances
    }

    fn average_embeddings(&self, chunk: &[Vec<f32>]) -> Vec<f32> {
        if chunk.is_empty() { return vec![]; }
        let dim = chunk[0].len();
        let mut avg = vec![0.0; dim];
        for emb in chunk {
            for k in 0..dim {
                avg[k] += emb[k];
            }
        }
        let count = chunk.len() as f32;
        for k in 0..dim {
            avg[k] /= count;
        }
        avg
    }
}

fn cosine_similarity_f32(v1: &[f32], v2: &[f32]) -> f32 {
    let mut dot_product = 0.0;
    let mut norm_v1 = 0.0;
    let mut norm_v2 = 0.0;
    for i in 0..v1.len() {
        dot_product += v1[i] * v2[i];
        norm_v1 += v1[i] * v1[i];
        norm_v2 += v2[i] * v2[i];
    }
    if norm_v1 <= 0.0 || norm_v2 <= 0.0 { return 0.0; }
    dot_product / (norm_v1.sqrt() * norm_v2.sqrt())
}

#[async_trait]
impl Transformation for SemanticSplitter {
    async fn transform(&self, nodes: Vec<Node>) -> Result<Vec<Node>> {
        let mut all_new_nodes = Vec::new();

        for node in nodes {
            if let crate::core::schema::NodeContent::Text(text) = &node.content {
                let regex = self.get_sentence_regex();
                let sentences: Vec<String> = regex.find_iter(text)
                    .map(|m| m.as_str().trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if sentences.len() <= 1 {
                    all_new_nodes.push(node);
                    continue;
                }

                const MAX_CHUNK_SENTENCES: usize = 1000;
                
                let all_embeddings = self.get_sentence_embeddings(&sentences).await?;
                let distances = self.calculate_distances(&all_embeddings);

                let mut sorted_distances = distances.clone();
                sorted_distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                
                let threshold = if !sorted_distances.is_empty() {
                    let threshold_idx = (sorted_distances.len() as f32 * self.breakpoint_percentile_threshold / 100.0) as usize;
                    *sorted_distances.get(threshold_idx.min(sorted_distances.len() - 1)).unwrap_or(&0.5)
                } else {
                    0.5
                };

                let mut current_chunk_text = Vec::new();
                for (j, sentence) in sentences.iter().enumerate() {
                    current_chunk_text.push(sentence.clone());
                    
                    let should_split = (j < distances.len() && distances[j] > threshold) || 
                                     (current_chunk_text.len() >= MAX_CHUNK_SENTENCES);

                    if should_split {
                        let mut new_node = Node::new_text(current_chunk_text.join(" "));
                        new_node.metadata = node.metadata.clone();
                        new_node.excluded_embed_metadata_keys = node.excluded_embed_metadata_keys.clone();
                        new_node.excluded_llm_metadata_keys = node.excluded_llm_metadata_keys.clone();
                        all_new_nodes.push(new_node);
                        current_chunk_text = Vec::new();
                    }
                }
                
                if !current_chunk_text.is_empty() {
                    let mut new_node = Node::new_text(current_chunk_text.join(" "));
                    new_node.metadata = node.metadata.clone();
                    new_node.excluded_embed_metadata_keys = node.excluded_embed_metadata_keys.clone();
                    new_node.excluded_llm_metadata_keys = node.excluded_llm_metadata_keys.clone();
                    all_new_nodes.push(new_node);
                }
            } else {
                all_new_nodes.push(node);
            }
        }

        Ok(all_new_nodes)
    }

    fn hash(&self) -> String {
        format!("semantic_splitter_{}", self.breakpoint_percentile_threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embeddings::base::Embedding;
    use async_trait::async_trait;
    use anyhow::Result;
    use half::f16;

    #[derive(Debug)]
    struct DummyEmbedding;
    #[async_trait]
    impl Embedding for DummyEmbedding {
        async fn get_text_embedding(&self, _text: &str) -> Result<Vec<f16>> { Ok(vec![]) }
        async fn get_text_embedding_batch(&self, _texts: Vec<String>, _batch_size: usize) -> Result<Vec<Vec<f16>>> { Ok(vec![]) }
        fn model_name(&self) -> &str { "dummy" }
    }

    #[test]
    fn test_calculate_distances_with_buffer() {
        let splitter = SemanticSplitter {
            embed_model: Arc::new(DummyEmbedding),
            buffer_size: 1,
            breakpoint_percentile_threshold: 95.0,
            embedding_batch_size: 32,
        };

        // Create 4 dummy embeddings
        let embeddings = vec![
            vec![1.0, 0.0],
            vec![1.0, 0.0],
            vec![0.0, 1.0], // Sudden change
            vec![0.0, 1.0],
        ];

        let distances = splitter.calculate_distances(&embeddings);
        
        // Expected buffer (size 1) averages:
        // avg[0] (0..1): [1.0, 0.0]
        // avg[1] (0..2): [2.0/3.0, 1.0/3.0]
        // avg[2] (1..3): [1.0/3.0, 2.0/3.0]
        // avg[3] (2..3): [0.0, 1.0]
        
        assert_eq!(distances.len(), 3);
        // Distances should be larger in the middle where the semantic shift happens
        assert!(distances[1] > distances[0]);
        assert!(distances[1] > distances[2]);
    }
}
