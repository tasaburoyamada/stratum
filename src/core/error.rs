use thiserror::Error;

#[derive(Error, Debug)]
pub enum StratumError {
    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Embedding error: {0}")]
    EmbeddingError(String),

    #[error("LLM error: {0}")]
    LlmError(String),

    #[error("Parsing error: {0}")]
    ParsingError(String),

    #[error("Index error: {0}")]
    IndexError(String),

    #[error("Lock poisoned: {0}")]
    LockPoisoned(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, StratumError>;
