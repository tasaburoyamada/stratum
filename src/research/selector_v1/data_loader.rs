use crate::feeding::SelectorTriplet;
use anyhow::Result;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct SelectorDataLoader {
    path: String,
}

impl SelectorDataLoader {
    pub fn new(path: &str) -> Self {
        Self { path: path.to_string() }
    }

    pub fn load_all(&self) -> Result<Vec<SelectorTriplet>> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut triplets = Vec::new();
        for line in reader.lines() {
            let line = line?;
            let triplet: SelectorTriplet = serde_json::from_str(&line)?;
            triplets.push(triplet);
        }
        Ok(triplets)
    }
}
