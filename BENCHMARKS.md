# Stratum Performance Benchmarks

This document details the performance metrics of Stratum compared to other popular RAG frameworks: **LlamaIndex**, **LangChain**, and **Rig**.

## Benchmark Environment
- **Data**: 100 Markdown files.
- **Model Emulation**: Simulated GPU latency (fixed 50ms per document embedding).
- **Hardware**: Same machine execution for all frameworks.
- **Framework Versions**:
    - Stratum: v0.4.0-dev (Rust)
    - Rig: v0.3.0 (Rust)
    - LlamaIndex: v0.14.x (Python)
    - LangChain: v0.3.x (Python)

## Results Summary

| Metric | **Stratum (Rust)** | **Rig (Rust)** | **LangChain (Python)** | **LlamaIndex (Python)** |
| :--- | :---: | :---: | :---: | :---: |
| **Ingestion Time (100 docs)** | **59 ms** | 51 ms | 51 ms | 663 ms |
| **Framework Overhead** | **9 ms** | 1 ms | 1 ms | 613 ms |
| **Query Latency (Top-1)** | **52 ms** | 52 ms | 51 ms | 54 ms |
| **Memory Usage (RSS)** | **28.7 MB** | 6.7 MB | 73.0 MB | 182.2 MB |
| **Startup Time (Cold Start)** | **< 1 ms** | < 1 ms | ~50 ms | ~1400 ms |

*Note: Ingestion/Query times include the fixed 50ms simulated model latency. "Overhead" is the time spent by the framework itself.*

## Key Insights

### 1. Minimal Overhead
Stratum exhibits a framework overhead of only **9ms** for processing 100 documents, whereas LlamaIndex requires over **600ms**. This reflects the efficiency of Rust's zero-cost abstractions.

### 2. Native Parallelism
While Python-based frameworks often default to sequential processing in common code examples, Stratum's ingestion pipeline is built on top of asynchronous streams. This allows for native batch processing, leading to significantly higher throughput in real-world scenarios with multiple CPU cores or GPU batching.

### 3. Memory & Functionality Balance
Stratum is slightly heavier than Rig (28.7 MB vs 6.7 MB) because it includes features necessary for production use:
- **ACID-compliant storage** (`redb`).
- **Sophisticated node parsing and chunking**.
- **Integrated document and index management**.
Despite this, it remains **6.3x more memory-efficient** than LlamaIndex.

### 4. Edge & Serverless Ready
With sub-millisecond startup times and low memory footprints, Stratum is ideal for:
- **AWS Lambda / Cloudflare Workers** (Fast cold starts).
- **Edge Devices** (Limited RAM).
- **High-throughput API services**.

## Reproducibility
The benchmark scripts used for this measurement are located in `stratum/benchmarks/`.
- `stratum_bench.rs`: Main measurement for Stratum.
- `llama_bench.py`: LlamaIndex measurement.
- `langchain_bench.py`: LangChain measurement.
- `rig_runner/`: Measurement for Rig.
