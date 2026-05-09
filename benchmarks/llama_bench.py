import time
import os
import psutil
import glob
import asyncio
from llama_index.core import VectorStoreIndex, Document, Settings
from llama_index.core.embeddings import BaseEmbedding
from typing import List

class AsyncMockEmbedding(BaseEmbedding):
    def _get_query_embedding(self, query: str) -> List[float]:
        time.sleep(0.05) 
        return [0.1] * 384
    def _get_text_embedding(self, text: str) -> List[float]:
        time.sleep(0.05)
        return [0.1] * 384
    def _get_text_embeddings(self, texts: List[str]) -> List[List[float]]:
        # Simulate batch processing (parallel)
        # In real GPU, this would be 50ms total for a batch
        time.sleep(0.05)
        return [[0.1] * 384 for _ in texts]
    async def _aget_query_embedding(self, query: str) -> List[float]:
        return self._get_query_embedding(query)
    async def _aget_text_embedding(self, text: str) -> List[float]:
        return self._get_text_embedding(text)
    async def _aget_text_embeddings(self, texts: List[str]) -> List[List[float]]:
        await asyncio.sleep(0.05)
        return [[0.1] * 384 for _ in texts]

def get_mem():
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / 1024 / 1024 # MB

async def main():
    t_total_start = time.perf_counter()
    
    # 1. Setup
    Settings.embed_model = AsyncMockEmbedding()
    Settings.llm = None 

    # 2. Data Loading
    data_path = "benchmarks/data_large"
    if not os.path.exists(data_path):
        data_path = "stratum/benchmarks/data_large"
    
    files = glob.glob(f"{data_path}/*.md")
    docs = []
    for filepath in files:
        with open(filepath, 'r', encoding='utf-8') as f:
            docs.append(Document(text=f.read()))
            
    # 3. Ingestion
    print(f"Ingesting {len(docs)} docs (Parallel Simulation)...")
    t_ingest_start = time.perf_counter()
    # LlamaIndex from_documents is sync by default but calls embed_model
    index = VectorStoreIndex.from_documents(docs)
    ingest_duration_ms = (time.perf_counter() - t_ingest_start) * 1000

    # 4. Query
    query_engine = index.as_query_engine()
    query_engine.query("Warmup")
    
    t_query_start = time.perf_counter()
    _response = query_engine.query("What is the advantage of using Rust and PyO3 together in Stratum?")
    query_duration_ms = (time.perf_counter() - t_query_start) * 1000

    mem = get_mem()
    total_duration_ms = (time.perf_counter() - t_total_start) * 1000
    
    print("--- LlamaIndex Results (Parallel Mock) ---")
    print(f"Ingestion Time (100 docs): {ingest_duration_ms:.2f} ms")
    print(f"Query Latency: {query_duration_ms:.2f} ms")
    print(f"Memory Usage (RSS): {mem:.2f} MB")
    print(f"Total Execution: {total_duration_ms:.2f} ms")

if __name__ == "__main__":
    asyncio.run(main())
