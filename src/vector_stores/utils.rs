use crate::vector_stores::types::{MetadataFilter, FilterOperator, FilterCondition};
use std::collections::HashMap;
use serde_json::Value;
use half::f16;

pub fn filter_metadata(
    metadata: &HashMap<String, Value>,
    filters: &Vec<MetadataFilter>,
    condition: &FilterCondition,
) -> bool {
    let results: Vec<bool> = filters.iter().map(|f| {
        if let Some(val) = metadata.get(&f.key) {
            match f.operator {
                FilterOperator::Eq => val == &f.value,
                FilterOperator::Ne => val != &f.value,
                FilterOperator::Gt => {
                    match (val, &f.value) {
                        (Value::Number(a), Value::Number(b)) => a.as_f64() > b.as_f64(),
                        _ => false,
                    }
                }
                FilterOperator::Lt => {
                    match (val, &f.value) {
                        (Value::Number(a), Value::Number(b)) => a.as_f64() < b.as_f64(),
                        _ => false,
                    }
                }
                FilterOperator::In => {
                    if let Value::Array(arr) = &f.value {
                        arr.contains(val)
                    } else {
                        false
                    }
                }
                FilterOperator::TextMatch => {
                    match (val, &f.value) {
                        (Value::String(a), Value::String(b)) => a.contains(b),
                        _ => false,
                    }
                }
            }
        } else {
            false
        }
    }).collect();

    if results.is_empty() {
        return true;
    }

    match condition {
        FilterCondition::And => results.iter().all(|&r| r),
        FilterCondition::Or => results.iter().any(|&r| r),
        FilterCondition::Not => !results.iter().all(|&r| r),
    }
}

pub fn cosine_similarity(v1: &[f16], v2: &[f16]) -> f32 {
    let mut dot_product: f32 = 0.0;
    let mut norm_v1: f32 = 0.0;
    let mut norm_v2: f32 = 0.0;

    for (a, b) in v1.iter().zip(v2.iter()) {
        let a_f = a.to_f32();
        let b_f = b.to_f32();
        dot_product += a_f * b_f;
        norm_v1 += a_f * a_f;
        norm_v2 += b_f * b_f;
    }

    if norm_v1 == 0.0 || norm_v2 == 0.0 {
        return 0.0;
    }

    dot_product / (norm_v1.sqrt() * norm_v2.sqrt())
}
