use serde::{Deserialize, Serialize};
use crate::core::schema::Node;
use half::f16;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    Eq, Gt, Lt, Ne, In, TextMatch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterCondition {
    And, Or, Not,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataFilter {
    pub key: String,
    pub value: serde_json::Value,
    pub operator: FilterOperator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataFilters {
    pub filters: Vec<MetadataFilter>,
    pub condition: FilterCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VectorStoreQueryMode {
    Default, Sparse, Hybrid, Mmr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreQuery {
    pub query_embedding: Option<Vec<f16>>,
    pub similarity_top_k: usize,
    pub filters: Option<MetadataFilters>,
    pub mode: VectorStoreQueryMode,
    pub alpha: Option<f32>,
}

pub struct VectorStoreQueryResult {
    pub nodes: Option<Vec<Node>>,
    pub similarities: Option<Vec<f32>>,
    pub ids: Option<Vec<String>>,
}
