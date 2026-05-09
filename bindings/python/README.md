# stratum-rag: Blazingly Fast RAG Engine for Python

**stratum-rag** is a high-performance Retrieval-Augmented Generation (RAG) engine powered by a Rust core. It provides blazingly fast document ingestion and semantic search capabilities for local LLM applications, with minimal overhead and memory footprint.

## 🚀 Why Stratum?

- **Extreme Performance**: Up to **60x lower overhead** compared to standard Python RAG frameworks.
- **Native Parallelism**: Built-in asynchronous ingestion pipeline for high throughput.
- **Local First**: Everything runs on your machine. No API keys, no hidden costs.
- **Memory Efficient**: ~1/6 the memory usage of traditional Python implementations.

## 🛠️ Quick Start

```python
import stratum

# 1. Initialize Stratum (In-memory or Persistent)
s = stratum.Stratum(persist_dir=None)

# 2. Ingest documents from a directory
# It automatically handles chunking and embedding using local models
s.ingest("./path/to/your/documents")

# 3. Perform semantic search
results = s.query("What are the advantages of Rust?", top_k=3)

for content, score in results:
    print(f"[{score:.4f}] {content[:100]}...")
```

## 📊 Benchmark (vs LlamaIndex)

| Metric | LlamaIndex | **Stratum-RAG** | Speedup |
| :--- | :--- | :--- | :--- |
| **Framework Overhead** | 613 ms | **9 ms** | **~68x** |
| **Cold Start** | ~1400 ms | **< 1 ms** | **~1400x** |
| **Memory (RSS)** | 182.2 MB | **28.7 MB** | **~6x lighter** |

## License
Apache-2.0
