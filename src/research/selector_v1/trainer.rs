use crate::research::selector_v1::model::VectorSelector;
use crate::embeddings::base::Embedding;
use crate::feeding::SelectorTriplet;
use candle_core::{Tensor, Device, Result};
use candle_nn::{VarMap, VarBuilder, Optimizer, AdamW, ParamsAdamW};
use std::sync::Arc;

pub struct SelectorTrainer {
    model: VectorSelector,
    varmap: VarMap,
    embed_model: Arc<dyn Embedding>,
    device: Device,
}

impl SelectorTrainer {
    pub fn new(dim: usize, embed_model: Arc<dyn Embedding>) -> Result<Self> {
        let device = Device::Cpu;
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, candle_core::DType::F32, &device);
        let model = VectorSelector::new(dim, vb)?;
        Ok(Self { model, varmap, embed_model, device })
    }

    pub async fn train_on_triplets(&mut self, triplets: Vec<SelectorTriplet>) -> Result<()> {
        let mut opt = AdamW::new(self.varmap.all_vars(), ParamsAdamW::default())?;

        for triplet in triplets {
            let q_emb = self.embed_model.get_text_embedding(&triplet.query).await
                .map_err(|e| candle_core::Error::Msg(e.to_string()))?;
            let p_emb = self.embed_model.get_text_embedding(&triplet.parent_context).await
                .map_err(|e| candle_core::Error::Msg(e.to_string()))?;

            let q_tensor = Tensor::from_vec(q_emb.iter().map(|&x| f32::from(x)).collect(), (1, q_emb.len()), &self.device)?;
            let p_tensor = Tensor::from_vec(p_emb.iter().map(|&x| f32::from(x)).collect(), (1, p_emb.len()), &self.device)?;

            for (i, choice) in triplet.choices.iter().enumerate() {
                let c_emb = self.embed_model.get_text_embedding(choice).await
                    .map_err(|e| candle_core::Error::Msg(e.to_string()))?;
                let c_tensor = Tensor::from_vec(c_emb.iter().map(|&x| f32::from(x)).collect(), (1, c_emb.len()), &self.device)?;

                let target = if triplet.selected_indices.contains(&i) { 1.0f32 } else { 0.0f32 };
                let target_tensor = Tensor::from_vec(vec![target], (1, 1), &self.device)?;

                // Forward pass
                let pred = self.model.forward(&q_tensor, &c_tensor, &p_tensor)?;
                
                // Manual BCE Loss: -(y * log(p) + (1-y) * log(1-p))
                let loss = ((target_tensor.clone() * pred.log()?)? + 
                          ((target_tensor.neg()? + 1.0)? * (pred.neg()? + 1.0)?.log()?)?)?.neg()?;
                let loss = loss.mean_all()?;
                
                // Backward and step
                opt.backward_step(&loss)?;
            }
        }
        Ok(())
    }
}
