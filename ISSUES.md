# Stratum Issue Tracker (Local)

This file tracks tasks and their status, mapped to Gitea issues.

| ID | Title | Status | Branch | Description |
|---|---|---|---|---|
| #1 | Implement Hierarchical Index (PageIndex) | Completed | develop | Create a semantic tree index that allows recursive summarization and non-vector retrieval. |
| #2 | LanceDB Alternative Stabilization | Completed | develop | Replaced LanceDB with pure-Rust HNSW + redb. |
| #3 | Local LLM Integration (Candle) | Completed | develop | Added CandleLlm for local inference. |
| #4 | Complete Hierarchical Index Persistence | Completed | develop | Ensure root node IDs are saved and index can be restored from storage. |
| #5 | Implement Hybrid Retriever | Completed | develop | Combine Vector, Keyword, and Hierarchical search strategies. |
| #6 | PDF Document Support | Completed | develop | Add a Reader for PDF files using a pure-Rust library. |

## Issue #4: Complete Hierarchical Index Persistence
### Tasks:
- [x] Extend `IndexStruct` or add metadata to support root node tracking.
- [x] Implement `HierarchicalIndex::from_storage_context` for restoration.
- [x] Add persistence test for Hierarchical Index.

## Issue #5: Hybrid Retriever
### Tasks:
- [x] Implement `HybridRetriever` struct and trait.
- [x] Support `WeightedSum` and `ReciprocalRankFusion` (RRF) modes.
- [x] Add integration tests for hybrid search.

## Issue #6: PDF Document Support
### Tasks:
- [x] Add `pdf-extract` dependency.
- [x] Implement `PdfReader` with tokio blocking task support.
- [x] Register `pdf` module in `lib.rs`.

