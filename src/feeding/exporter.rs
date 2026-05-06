use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;

pub struct DatasetExporter;

impl DatasetExporter {
    pub fn export_jsonl<T: serde::Serialize>(items: &[T], path: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        for item in items {
            let json = serde_json::to_string(item)?;
            writeln!(file, "{}", json)?;
        }
        Ok(())
    }
}
