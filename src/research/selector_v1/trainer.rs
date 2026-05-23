use crate::research::selector_v1::model::VectorSelector;
use candle_core::{Device, Result, Tensor};
use candle_nn::{AdamW, Optimizer, ParamsAdamW, VarBuilder, VarMap};

pub struct SelectorTrainer {
    model: VectorSelector,
    varmap: VarMap,
    _device: Device,
}

impl SelectorTrainer {
    pub fn new(dim: usize) -> Result<Self> {
        let device = Device::Cpu;
        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, candle_core::DType::F32, &device);
        let model = VectorSelector::new(dim, vb)?;
        Ok(Self { model, varmap, _device: device })
    }

    pub fn train_on_tensors(&mut self, queries: &Tensor, choices: &Tensor, parents: &Tensor, targets: &Tensor, epochs: usize) -> Result<()> {
        let mut opt = AdamW::new(self.varmap.all_vars(), ParamsAdamW::default())?;
        
        let batch_size = queries.dim(0)?;
        println!("🏋️ Training on batch of {} samples for {} epochs...", batch_size, epochs);

        for epoch in 0..epochs {
            // Forward pass
            let pred = self.model.forward(queries, choices, parents)?;
            
            // Manual BCE Loss: -(y * log(p) + (1-y) * log(1-p))
            // Adding a small epsilon 1e-7 to prevent log(0)
            let term1 = (targets.clone() * (pred.clone() + 1e-7)?.log()?)?;
            let term2 = ((targets.neg()? + 1.0)? * ((pred.neg()? + 1.0)? + 1e-7)?.log()?)?;
            
            let loss = (term1 + term2)?.neg()?;
            let loss = loss.mean_all()?;
            
            // Backward and step
            opt.backward_step(&loss)?;
            
            if (epoch + 1) % 10 == 0 || epoch == epochs - 1 {
                println!("Epoch {}/{} - Loss: {:?}", epoch + 1, epochs, loss.to_vec0::<f32>()?);
            }
        }
        Ok(())
    }

    pub fn save(&self, path: &str) -> Result<()> {
        self.varmap.save(path)?;
        println!("💾 Model weights saved to {}", path);
        Ok(())
    }
}
