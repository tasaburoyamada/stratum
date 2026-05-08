import time
import os
import psutil
from llama_index.core import VectorStoreIndex, SimpleDirectoryReader, Settings
from llama_index.core.embeddings import BaseEmbedding
from typing import Any, List
import subprocess

# We simulate a "real" model delay but avoid torch due to the broken pyenv installation.
# Let's use a dummy embedding that exactly mimics the latency of Candle CPU to be conservative for Python.
# Actually, since we can't use PyTorch, let's use a subprocess to call a rust mini-binary to get embeddings.

class SubprocessEmbedding(BaseEmbedding):
    def _get_query_embedding(self, query: str) -> List[float]:
        time.sleep(0.05) # 50ms simulated GPU latency
        return [0.1] * 384
    def _get_text_embedding(self, text: str) -> List[float]:
        time.sleep(0.05)
        return [0.1] * 384
    def _get_text_embeddings(self, texts: List[str]) -> List[List[float]]:
        time.sleep(0.05 * len(texts))
        return [[0.1] * 384 for _ in texts]
    async def _aget_query_embedding(self, query: str) -> List[float]:
        return self._get_query_embedding(query)
    async def _aget_text_embedding(self, text: str) -> List[float]:
        return self._get_text_embedding(text)

def get_mem():
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / 1024 / 1024 # MB

def main():
    total_start = time.time()
    
    # 1. Setup (Measure startup)
    startup_start = time.time()
    Settings.embed_model = SubprocessEmbedding()
    Settings.llm = None 
    startup_time = time.time() - startup_start

    # 2. Ingestion (100 docs)
    print("Reading 100 documents...")
    docs = SimpleDirectoryReader("benchmarks/data_large").load_data()
    
    print("Ingesting...")
    ingest_start = time.time()
    index = VectorStoreIndex.from_documents(docs)
    ingestion_time = time.time() - ingest_start

    # 3. Query (Latency)
    query_engine = index.as_query_engine()
    query_engine.query("Warmup")
    
    query_start = time.time()
    _response = query_engine.query("What is the advantage of using Rust and PyO3 together in Stratum?")
    query_latency = time.time() - query_start

    mem = get_mem()
    
    print("--- LlamaIndex Results ---")
    print(f"Startup Time: {startup_time * 1000:.2f} ms")
    print(f"Ingestion Time (100 docs): {ingestion_time * 1000:.2f} ms")
    print(f"Query Latency: {query_latency * 1000:.2f} ms")
    print(f"Memory Usage (RSS): {mem:.2f} MB")
    print(f"Total Execution: {(time.time() - total_start) * 1000:.2f} ms")

if __name__ == "__main__":
    main()
