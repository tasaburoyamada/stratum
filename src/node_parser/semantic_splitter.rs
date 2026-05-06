use crate::core::schema::Node;
use crate::core::ingestion::transformation::Transformation;
use crate::embeddings::base::Embedding;
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;
use std::sync::Arc;

pub struct SemanticSplitter {
    pub embed_model: Arc<dyn Embedding>,
    pub buffer_size: usize,
    pub breakpoint_percentile_threshold: f32,
    pub sentence_regex: Regex,
}

impl std::fmt::Debug for SemanticSplitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SemanticSplitter")
            .field("buffer_size", &self.buffer_size)
            .field("breakpoint_percentile_threshold", &self.breakpoint_percentile_threshold)
            .finish()
    }
}

impl SemanticSplitter {
    pub fn new(embed_model: Arc<dyn Embedding>) -> Self {
        Self {
            embed_model,
            buffer_size: 1,
            breakpoint_percentile_threshold: 95.0,
            sentence_regex: Regex::new(r"[^.!?。！？]+[.!?。！？]?").unwrap(),
        }
    }

    async fn get_sentence_embeddings(&self, sentences: &[String]) -> Result<Vec<Vec<f32>>> {
        let embeddings = self.embed_model.get_text_embedding_batch(sentences.to_vec(), 32).await?;
        Ok(embeddings.into_iter().map(|e| e.iter().map(|&x| f32::from(x)).collect()).collect())
    }

    fn calculate_distances(&self, embeddings: &[Vec<f32>]) -> Vec<f32> {
        let mut distances = Vec::new();
        if embeddings.len() < 2 {
            return distances;
        }

        let mut combined_embeddings = Vec::new();
        for i in 0..embeddings.len() {
            let start = i.saturating_sub(self.buffer_size);
            let end = (i + self.buffer_size).min(embeddings.len() - 1);
            
            let mut avg_emb = vec![0.0; embeddings[0].len()];
            let count = (end - start + 1) as f32;
            for j in start..=end {
                for k in 0..avg_emb.len() {
                    avg_emb[k] += embeddings[j][k];
                }
            }
            for k in 0..avg_emb.len() {
                avg_emb[k] /= count;
            }
            combined_embeddings.push(avg_emb);
        }

        for i in 0..combined_embeddings.len() - 1 {
            let dist = 1.0 - cosine_similarity_f32(&combined_embeddings[i], &combined_embeddings[i+1]);
            distances.push(dist);
        }
        distances
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
                let sentences: Vec<String> = self.sentence_regex.find_iter(text)
                    .map(|m| m.as_str().trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if sentences.len() <= 1 {
                    all_new_nodes.push(node);
                    continue;
                }

                let embeddings = self.get_sentence_embeddings(&sentences).await?;
                let distances = self.calculate_distances(&embeddings);

                let mut sorted_distances = distances.clone();
                sorted_distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
                
                let threshold_idx = (sorted_distances.len() as f32 * self.breakpoint_percentile_threshold / 100.0) as usize;
                let threshold = sorted_distances.get(threshold_idx.min(sorted_distances.len() - 1)).cloned().unwrap_or(0.5);

                let mut chunks = Vec::new();
                let mut current_chunk = Vec::new();

                for (i, sentence) in sentences.into_iter().enumerate() {
                    current_chunk.push(sentence);
                    if i < distances.len() && distances[i] > threshold {
                        chunks.push(current_chunk.join(" "));
                        current_chunk = Vec::new();
                    }
                }
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk.join(" "));
                }

                for chunk in chunks {
                    let mut new_node = Node::new_text(chunk);
                    new_node.metadata = node.metadata.clone();
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
            sentence_regex: Regex::new(r"[^.!?。！？]+[.!?。！？]?").unwrap(),
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
