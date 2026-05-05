# Stratum Issue Tracker (Local)

This file tracks tasks and their status, mapped to Gitea issues.

| ID | Title | Status | Branch | Description |
|---|---|---|---|---|
| #1 | Implement Hierarchical Index (PageIndex) | In Progress | feature/1-hierarchical-index | Create a semantic tree index that allows recursive summarization and non-vector retrieval. |
| #2 | LanceDB Alternative Stabilization | Completed | develop | Replaced LanceDB with pure-Rust HNSW + redb. |
| #3 | Local LLM Integration (Candle) | Completed | develop | Added CandleLlm for local inference. |

## Issue #1: Hierarchical Index (PageIndex)
### Tasks:
- [x] Define Hierarchical Node Structure (Parent/Child relations)
- [x] Implement `HierarchicalIndex` trait and struct
- [x] Implement recursive summarization logic (using LLM)
- [x] Implement `HierarchicalRetriever` (Semantic traversal)
- [x] Add integration tests for Hierarchical RAG
