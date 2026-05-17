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
| #13 | Salesforce Exam Ingestion | Completed | develop | Implement a dedicated reader or JSONL ingestion flow for Salesforce exam data. |
| #14 | Hybrid LLM Support (OpenAI API) | Completed | develop | Implement an OpenAI-compatible client to support cloud-hosted models. |
| #15 | In-memory Storage Support | Completed | develop | Implement runtime switching between persistent redb and high-speed in-memory storage. |

## Issue #15: In-memory Storage Support
- [x] Implement `StorageContext::in_memory()` constructor to initialize non-persistent store implementations.
- [x] Add runtime configuration/flag to switch between `redb` (Long-term) and `Simple` (Short-term/Task-local) memory.
- [x] Optimization: Introduce build-time feature flags (e.g., `--features memory-only`) to exclude `redb` dependencies for restricted environments like WASM.
- [x] Strategy: Align with "Stratification" concept, where in-memory serves as a high-speed scratchpad and redb serves as the durable knowledge base.

## Issue #9: Dedicated Small Model Research
- [x] Define MLP-based `VectorSelector` architecture in Candle.
- [x] Implement `SelectorTrainer` for intelligence distillation.
- [x] Build research prototype script for training proof-of-concept.
- [ ] Large-scale training and evaluation on captured data.
- [ ] Integration into `HierarchicalRetriever` as an alternative to LLM.

## Issue #13: Salesforce Exam Ingestion
- [x] Implement `RefineryParser` (LLM-based structured extraction).
- [x] Integrate `DATA_SPEC.md` into ingestion pipeline.
- [x] Enable NGA-to-RAG knowledge refinement path.

## Issue #14: Hybrid LLM Support
- [x] Implement `OpenAIClient` with streaming support.
- [x] Support custom base URLs (Ollama/VLLM).
- [x] Integrate into `LlmClient` trait architecture.

---

# DeepSeek Adversarial Audit Results (2026-05-17)

## 🔴 CRITICAL ARCHITECTURAL FINDINGS

### [STRATUM-CRITICAL-01] Identity Crisis: Stratum vs LlamaIndex
- **Issue**: The project claims to be both a "Deterministic HV-CAD Engine" and a "Stochastic LlamaIndex Engine".
- **Risk**: Conflicting design decisions in query routing and state management.
- **Action**: Unified project identity required.

### [STRATUM-CRITICAL-02] Physical Performance Contradiction
- **Issue**: Benchmark claims (9ms overhead, <1ms cold start, 28MB RSS) are physically impossible given the ML frameworks used (Candle/BERT).
- **Risk**: Trust failure; benchmarks are currently "hallucinated" or based on an incomplete pipeline.
- **Action**: Transparent benchmarking with reproducible scripts.

### [STRATUM-CRITICAL-03] Missing HV-CAD Infrastructure
- **Issue**: No actual implementation of L1/L2/L3 layers despite documentation claims. `.vlog` integration is a stub.
- **Risk**: The project remains a standard RAG library rather than an HV-CAD component.
- **Action**: Implement `.vlog` parser and dynamic bias weighting in retrieval.

### [STRATUM-CRITICAL-04] Stratification Missing
- **Issue**: "Stratification" (time/confidence/importance layering) is absent in code.
- **Risk**: Fundamental architectural promise is unfulfilled.
- **Action**: Implement temporal decay and confidence-weighted retrieval layers.

## 🟡 TECHNICAL DEBT & FLAWS

- **[ST-01] Redb vs LanceDB Confusion**: Conflicting storage strategies for vector vs metadata.
- **[ST-02] Over-engineering**: 7 levels of module nesting for basic I/O tasks.
- **[ST-03] Missing Stream Refining**: Ingestion is batch-oriented, contradicting the "Zero-Copy Pipeline" goal.
- **[ST-04] Trait Erasure vs Type Safety**: Claim of "Type-Safe Ingestion" is invalidated by heavy use of dynamic dispatch (`Box<dyn Reader>`).
