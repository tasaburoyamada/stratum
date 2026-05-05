use crate::feeding::triplet::DatasetTriplet;
use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;

pub struct DatasetExporter;

impl DatasetExporter {
    pub fn export_jsonl(triplets: &[DatasetTriplet], path: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        for triplet in triplets {
            let json = serde_json::to_string(triplet)?;
            writeln!(file, "{}", json)?;
        }
        Ok(())
    }
}
