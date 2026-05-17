use crate::indices::hierarchical::selector::NodeSelector;
use crate::embeddings::base::Embedding;
use crate::core::schema::Node;
use async_trait::async_trait;
use anyhow::Result;
use std::sync::Arc;
use candle_core::{Tensor, Device, Module};
use candle_nn::{Linear, VarBuilder, ops};

/// The neural network model for the Vector Selector.
pub struct VectorSelectorModel {
    fc1: Linear,
    fc2: Linear,
    fc3: Linear,
}

impl VectorSelectorModel {
    pub fn new(dim: usize, vb: VarBuilder) -> candle_core::Result<Self> {
        let fc1 = candle_nn::linear(dim * 3, 512, vb.pp("fc1"))?;
        let fc2 = candle_nn::linear(512, 128, vb.pp("fc2"))?;
        let fc3 = candle_nn::linear(128, 1, vb.pp("fc3"))?;
        Ok(Self { fc1, fc2, fc3 })
    }

    pub fn forward(&self, q: &Tensor, c: &Tensor, p: &Tensor) -> candle_core::Result<Tensor> {
        let x = Tensor::cat(&[q, c, p], 1)?;
        let x = self.fc1.forward(&x)?;
        let x = x.relu()?;
        let x = self.fc2.forward(&x)?;
        let x = x.relu()?;
        let x = self.fc3.forward(&x)?;
        ops::sigmoid(&x)
    }
}

/// The NodeSelector implementation that uses the trained VectorSelectorModel.
pub struct VectorNodeSelector {
    model: VectorSelectorModel,
    embed_model: Arc<dyn Embedding>,
    device: Device,
    threshold: f32,
}

impl VectorNodeSelector {
    /// Initialize with loaded weights and an embedding model.
    pub fn new(weights_path: &str, dim: usize, embed_model: Arc<dyn Embedding>, threshold: f32) -> Result<Self> {
        let device = Device::Cpu;
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_path], candle_core::DType::F32, &device)? };
        let model = VectorSelectorModel::new(dim, vb)?;
        
        Ok(Self {
            model,
            embed_model,
            device,
            threshold,
        })
    }
}

#[async_trait]
impl NodeSelector for VectorNodeSelector {
    async fn select(
        &self,
        query_str: &str,
        parent_node: &Node,
        children_nodes: &[Node],
    ) -> Result<Vec<usize>> {
        let q_emb = self.embed_model.get_text_embedding(query_str).await?;
        let parent_content = parent_node.get_content(None)?;
        let p_emb = self.embed_model.get_text_embedding(&parent_content).await?;
        
        let q_tensor = Tensor::from_vec(q_emb.iter().map(|&x| f32::from(x)).collect::<Vec<_>>(), (1, q_emb.len()), &self.device)?;
        let p_tensor = Tensor::from_vec(p_emb.iter().map(|&x| f32::from(x)).collect::<Vec<_>>(), (1, p_emb.len()), &self.device)?;

        let mut selected_indices = Vec::new();

        for (i, child) in children_nodes.iter().enumerate() {
            let child_content = child.get_content(None)?;
            let c_emb = self.embed_model.get_text_embedding(&child_content).await?;
            let c_tensor = Tensor::from_vec(c_emb.iter().map(|&x| f32::from(x)).collect::<Vec<_>>(), (1, c_emb.len()), &self.device)?;

            // Inference
            let score_tensor = self.model.forward(&q_tensor, &c_tensor, &p_tensor)?;
            let score = score_tensor.get(0)?.get(0)?.to_scalar::<f32>()?;

            if score >= self.threshold {
                selected_indices.push(i);
            }
        }

        Ok(selected_indices)
    }
}
