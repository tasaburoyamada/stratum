use crate::vector_stores::types::{MetadataFilter, FilterOperator, FilterCondition};
use serde_json::Value;
use half::f16;

use crate::core::schema::TypedMetadata;

pub fn filter_metadata(
    metadata: &TypedMetadata,
    filters: &Vec<MetadataFilter>,
    condition: &FilterCondition,
) -> bool {
    let results: Vec<bool> = filters.iter().map(|f| {
        // 1. Try structured fields first
        let val_opt = match f.key.as_str() {
            "url" => metadata.url.as_ref().map(|s| Value::String(s.clone())),
            "file_path" => metadata.file_path.as_ref().map(|s| Value::String(s.clone())),
            "genre" => metadata.genre.as_ref().map(|s| Value::String(s.clone())),
            "timestamp" => metadata.timestamp.as_ref().map(|t| Value::String(t.to_rfc3339())),
            _ => metadata.extra.get(&f.key).cloned(),
        };

        if let Some(val) = val_opt {
            match f.operator {
                FilterOperator::Eq => val == f.value,
                FilterOperator::Ne => val != f.value,
                FilterOperator::Gt => {
                    match (&val, &f.value) {
                        (Value::Number(a), Value::Number(b)) => a.as_f64() > b.as_f64(),
                        _ => false,
                    }
                }
                FilterOperator::Lt => {
                    match (&val, &f.value) {
                        (Value::Number(a), Value::Number(b)) => a.as_f64() < b.as_f64(),
                        _ => false,
                    }
                }
                FilterOperator::In => {
                    if let Value::Array(arr) = &f.value {
                        arr.contains(&val)
                    } else {
                        false
                    }
                }
                FilterOperator::TextMatch => {
                    match (&val, &f.value) {
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
    if v1.len() != v2.len() || v1.is_empty() {
        return 0.0;
    }

    let mut dot_product = 0.0;
    let mut norm_v1 = 0.0;
    let mut norm_v2 = 0.0;

    // Hint compiler for auto-vectorization: ensure length is checked and use simple indexing
    let len = v1.len();
    for i in 0..len {
        let a = v1[i].to_f32();
        let b = v2[i].to_f32();
        dot_product += a * b;
        norm_v1 += a * a;
        norm_v2 += b * b;
    }

    if norm_v1 <= 0.0 || norm_v2 <= 0.0 {
        return 0.0;
    }

    dot_product / (norm_v1.sqrt() * norm_v2.sqrt())
}
