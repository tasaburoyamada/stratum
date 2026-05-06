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
| #8 | Intelligence Feeding | Completed | develop | Implement feeding.rs to export structured datasets for LoRA/distillation. |
| #9 | Dedicated Small Model Research | In Progress | develop | Explore training a specialized router/selector model for PageIndex. |
| #10 | Robust RAG Storage in Lasada | Completed | develop | Move RAG state to StratumRAG bridge and implement proper persistence. |
| #11 | RAG Evaluation Module | Completed | develop | Implement Faithfulness and Relevancy evaluators. |
| #12 | Semantic Chunking Parser | Completed | develop | Implement intelligent content splitting based on embedding shifts. |
| #13 | Salesforce Exam Ingestion | To Do | feature/13-salesforce-exam | Implement a dedicated reader or JSONL ingestion flow for Salesforce exam data. |

## Issue #9: Dedicated Small Model Research
- [x] Define MLP-based `VectorSelector` architecture in Candle.
- [x] Implement `SelectorTrainer` for intelligence distillation.
- [x] Build research prototype script for training proof-of-concept.
- [ ] Large-scale training and evaluation on captured data.
- [ ] Integration into `HierarchicalRetriever` as an alternative to LLM.

## Issue #11: RAG Evaluation Module
- [x] Define `BaseEvaluator` trait and `EvaluationResult` schema.
- [x] Implement `FaithfulnessEvaluator` (Hallucination detection).
- [x] Implement `RelevancyEvaluator` (Query alignment).
- [x] Add automated evaluation tests.

## Issue #12: Semantic Chunking Parser
- [x] Implement `SemanticSplitter` with dynamic boundary detection.
- [x] Use cosine similarity between sentence embeddings to find breakpoints.
- [x] Integrate into `Transformation` pipeline.

## Issue #13: Salesforce Exam Ingestion
### Tasks:
- [ ] Implement `SalesforceJsonReader` (specialized schema handling).
- [ ] Add integration for NGA-captured raw visual data.
- [ ] Test hybrid search with complex exam questions.
