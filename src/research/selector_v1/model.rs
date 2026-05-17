use candle_core::{Tensor, Result, Module};
use candle_nn::{Linear, VarBuilder, ops};

pub struct VectorSelector {
    fc1: Linear,
    fc2: Linear,
    fc3: Linear,
}

impl VectorSelector {
    pub fn new(dim: usize, vb: VarBuilder) -> Result<Self> {
        let fc1 = candle_nn::linear(dim * 3, 512, vb.pp("fc1"))?;
        let fc2 = candle_nn::linear(512, 128, vb.pp("fc2"))?;
        let fc3 = candle_nn::linear(128, 1, vb.pp("fc3"))?;
        Ok(Self { fc1, fc2, fc3 })
    }

    pub fn forward(&self, q: &Tensor, c: &Tensor, p: &Tensor) -> Result<Tensor> {
        let x = Tensor::cat(&[q, c, p], 1)?;
        let x = self.fc1.forward(&x)?;
        let x = x.relu()?;
        let x = self.fc2.forward(&x)?;
        let x = x.relu()?;
        let x = self.fc3.forward(&x)?;
        ops::sigmoid(&x)
    }
}
