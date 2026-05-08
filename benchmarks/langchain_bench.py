import time
import os
import psutil
from langchain_core.embeddings import Embeddings
from langchain_core.vectorstores import InMemoryVectorStore
from langchain_community.document_loaders import DirectoryLoader

class MockEmbeddings(Embeddings):
    def embed_documents(self, texts):
        time.sleep(0.05) # Simulated GPU latency per batch
        return [[0.1]*384 for _ in texts]
    def embed_query(self, text):
        time.sleep(0.05)
        return [0.1]*384

def get_mem():
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / 1024 / 1024 # MB

def main():
    total_start = time.time()
    
    # 1. Setup
    startup_start = time.time()
    embed_model = MockEmbeddings()
    startup_time = time.time() - startup_start

    # 2. Ingestion
    print("Reading 100 documents...")
    import glob
    from langchain_core.documents import Document
    docs = []
    for filepath in glob.glob("benchmarks/data_large/**/*.md", recursive=True):
        with open(filepath, 'r', encoding='utf-8') as f:
            docs.append(Document(page_content=f.read(), metadata={"source": filepath}))
    
    print("Ingesting...")
    ingest_start = time.time()
    vectorstore = InMemoryVectorStore.from_documents(docs, embed_model)
    ingestion_time = time.time() - ingest_start

    # 3. Query
    # Warmup
    vectorstore.similarity_search("Warmup", k=1)
    
    query_start = time.time()
    _response = vectorstore.similarity_search("What is the advantage of using Rust and PyO3 together in Stratum?", k=1)
    query_latency = time.time() - query_start

    mem = get_mem()
    
    print("--- LangChain Results ---")
    print(f"Startup Time: {startup_time * 1000:.2f} ms")
    print(f"Ingestion Time (100 docs): {ingestion_time * 1000:.2f} ms")
    print(f"Query Latency: {query_latency * 1000:.2f} ms")
    print(f"Memory Usage (RSS): {mem:.2f} MB")
    print(f"Total Execution: {(time.time() - total_start) * 1000:.2f} ms")

if __name__ == "__main__":
    main()
