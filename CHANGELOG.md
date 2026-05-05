# Changelog - Stratum

All notable changes to this project will be documented in this file.

## [0.3.0] - 2026-05-06

### Added
- **PDF Support**: Added `PdfReader` for extracting text from PDF documents using pure-Rust `pdf-extract`.
- **Hybrid Retriever**: New `HybridRetriever` supporting `WeightedSum` and `ReciprocalRankFusion` (RRF) for combining multiple search strategies.
- **Hierarchical Index (PageIndex)**: Implemented a recursive summarization-based index that enables semantic traversal without vector embeddings.
- **Hierarchical Persistence**: Ensured hierarchical indexes can be fully restored from storage with root node tracking.

### Changed
- **Index Management**: Integrated `IndexStore` into `VectorStoreIndex` for automated metadata tracking.
- **Improved Metadata**: Added `extra` fields to `IndexStruct` for custom index-specific metadata.

## [0.2.0] - 2026-05-06

### Added
- **NativeVectorStore**: Implemented a pure-Rust persistent vector store using `hnsw_rs` for indexing and `redb` for ACID storage.
- **Local LLM Support (Candle)**: Added `CandleLlm` implementing the `LlmClient` trait, allowing local inference with Llama-3 models.
- **Storage Persistence**: Added `load` methods to all storage components and `StorageContext::from_dir` for easy state restoration.
- **Structured Data Support**: Added `JsonReader` for importing data from JSON files and arrays.
- **Enhanced Filtering**: Added `KeywordPostprocessor` for hybrid RAG filtering based on required/excluded keywords.
- **File Filtering**: Added extension-based filtering to `SimpleDirectoryReader`.
- **Index Management**: Integrated `IndexStore` into `VectorStoreIndex` for automated metadata tracking.

### Changed
- **Renamed Project**: Renamed the crate from `llama-index-rust` to `stratum` for clarity and uniqueness.
- **Optimized Tokenization**: Refactored `SentenceSplitter` to use a cached token counting strategy ($O(n^2) \rightarrow O(n)$).
- **Decoupled LLM**: Moved `LlmClient` to a dedicated `llm` module for better architectural separation.

### Fixed
- **Safety**: Added safetensors header validation before memory mapping in `CandleEmbedding` to prevent crashes on invalid files.
- **Consistency**: Updated all tests and workspace dependencies to reflect the new crate name.

---
## [0.1.0] - 2026-04-28
- Initial internal release.
- Basic RAG pipeline implementation.
