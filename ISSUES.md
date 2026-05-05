# Stratum Issue Tracker (Local)

This file tracks tasks and their status, mapped to Gitea issues.

| ID | Title | Status | Branch | Description |
|---|---|---|---|---|
| #1 | Implement Hierarchical Index (PageIndex) | Completed | develop | Create a semantic tree index that allows recursive summarization and non-vector retrieval. |
| #2 | LanceDB Alternative Stabilization | Completed | develop | Replaced LanceDB with pure-Rust HNSW + redb. |
| #3 | Local LLM Integration (Candle) | Completed | develop | Added CandleLlm for local inference. |
| #4 | Complete Hierarchical Index Persistence | In Progress | feature/4-hierarchical-persistence | Ensure root node IDs are saved and index can be restored from storage. |
| #5 | Implement Hybrid Retriever | To Do | - | Combine Vector, Keyword, and Hierarchical search strategies. |
| #6 | PDF Document Support | To Do | - | Add a Reader for PDF files using a pure-Rust library. |

## Issue #1: Hierarchical Index (PageIndex)
...
- [x] Add integration tests for Hierarchical RAG

## Issue #4: Complete Hierarchical Index Persistence
### Tasks:
- [ ] Extend `IndexStruct` or add metadata to support root node tracking.
- [ ] Implement `HierarchicalIndex::from_storage_context` for restoration.
- [ ] Add persistence test for Hierarchical Index.

