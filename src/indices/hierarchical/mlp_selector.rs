use crate::indices::hierarchical::selector::NodeSelector;
use crate::research::selector_v1::model::VectorSelector;
use crate::core::schema::Node;
use candle_core::Device;
use candle_nn::VarBuilder;
use anyhow::Result;
use async_trait::async_trait;

pub struct MlpNodeSelector {
    model: VectorSelector,
}

impl MlpNodeSelector {
    pub fn new() -> Result<Self> {
        let device = Device::Cpu;
        let mut varmap = candle_nn::VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, candle_core::DType::F32, &device);
        let model = VectorSelector::new(384, vb)?;
        // Load weights
        varmap.load("src/research/selector_v1/selector_weights.safetensors")?;
        Ok(Self { model })
    }
}

#[async_trait]
impl NodeSelector for MlpNodeSelector {
    async fn select(
        &self,
        _query_str: &str,
        _parent_node: &Node,
        children_nodes: &[Node],
    ) -> Result<Vec<usize>> {
        let mut selected = Vec::new();
        for (i, _) in children_nodes.iter().enumerate() {
            selected.push(i);
        }
        Ok(selected)
    }
}
