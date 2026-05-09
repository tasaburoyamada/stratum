import time
import os
import psutil
import glob
from langchain_core.embeddings import Embeddings
from langchain_core.vectorstores import InMemoryVectorStore
from langchain_core.documents import Document

class ParallelMockEmbeddings(Embeddings):
    def embed_documents(self, texts):
        # Simulate batch processing (parallel)
        time.sleep(0.05)
        return [[0.1]*384 for _ in texts]
    def embed_query(self, text):
        time.sleep(0.05)
        return [0.1]*384

def get_mem():
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / 1024 / 1024 # MB

def main():
    t_total_start = time.perf_counter()
    
    # 1. Setup
    embed_model = ParallelMockEmbeddings()

    # 2. Ingestion
    docs = []
    data_path = "benchmarks/data_large"
    if not os.path.exists(data_path):
        data_path = "stratum/benchmarks/data_large"
        
    for filepath in glob.glob(f"{data_path}/*.md", recursive=True):
        with open(filepath, 'r', encoding='utf-8') as f:
            docs.append(Document(page_content=f.read()))
    
    print(f"Ingesting {len(docs)} docs (Parallel Simulation)...")
    t_ingest_start = time.perf_counter()
    vectorstore = InMemoryVectorStore.from_documents(docs, embed_model)
    ingest_duration_ms = (time.perf_counter() - t_ingest_start) * 1000

    # 3. Query
    vectorstore.similarity_search("Warmup", k=1)
    
    t_query_start = time.perf_counter()
    _response = vectorstore.similarity_search("What is the advantage of using Rust and PyO3 together in Stratum?", k=1)
    query_duration_ms = (time.perf_counter() - t_query_start) * 1000

    mem = get_mem()
    total_duration_ms = (time.perf_counter() - t_total_start) * 1000
    
    print("--- LangChain Results (Parallel Mock) ---")
    print(f"Ingestion Time (100 docs): {ingest_duration_ms:.2f} ms")
    print(f"Query Latency: {query_duration_ms:.2f} ms")
    print(f"Memory Usage (RSS): {mem:.2f} MB")
    print(f"Total Execution: {total_duration_ms:.2f} ms")

if __name__ == "__main__":
    main()
