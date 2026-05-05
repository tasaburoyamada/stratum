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
| #7 | Integration with Lasada | Completed | develop | Replace/Augment Lasada's RAG capabilities with Stratum. |
| #8 | Intelligence Feeding | In Progress | feature/8-feeding-layer | Implement feeding.rs to export structured datasets for LoRA/distillation. |
| #9 | Dedicated Small Model Research | To Do | - | Explore training a specialized router/selector model for PageIndex. |

## Issue #1: Hierarchical Index (PageIndex)
- [x] Define Hierarchical Node Structure (Parent/Child relations)
- [x] Implement `HierarchicalIndex` trait and struct
- [x] Implement recursive summarization logic (using LLM)
- [x] Implement `HierarchicalRetriever` (Semantic traversal)
- [x] Add integration tests for Hierarchical RAG

## Issue #4: Complete Hierarchical Index Persistence
- [x] Extend `IndexStruct` or add metadata to support root node tracking.
- [x] Implement `HierarchicalIndex::from_storage_context` for restoration.
- [x] Add persistence test for Hierarchical Index.

## Issue #5: Hybrid Retriever
- [x] Implement `HybridRetriever` struct and trait.
- [x] Support `WeightedSum` and `ReciprocalRankFusion` (RRF) modes.
- [x] Add integration tests for hybrid search.

## Issue #6: PDF Document Support
- [x] Add `pdf-extract` dependency.
- [x] Implement `PdfReader` with tokio blocking task support.
- [x] Register `pdf` module in `lib.rs`.

## Issue #7: Integration with Lasada
- [x] Define `StratumRAG` bridge in `lasada`.
- [x] Implement data synchronization/ingestion from `lasada` context to Stratum.
- [x] Expose Stratum search results to `lasada` agents.
- [x] Fix compilation errors and API alignment in `lasada/src/core/interpreter.rs`.

## Issue #8: Intelligence Feeding
### Tasks:
- [ ] Implement `feeding.rs` module in Stratum.
- [ ] Define `DatasetTriplet` structure (Query, Context, Response).
- [ ] Create a mechanism to capture and store these triplets during RAG operation.
- [ ] Implement export to HuggingFace-compatible JSONL format.
