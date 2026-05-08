import time
import os
import psutil
from llama_index.core import VectorStoreIndex, Document, Settings
from llama_index.embeddings.huggingface import HuggingFaceEmbedding

def get_mem():
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / 1024 / 1024 # MB

def main():
    total_start = time.time()
    
    # 1. Setup (Measure startup)
    startup_start = time.time()
    # Use local model
    Settings.embed_model = HuggingFaceEmbedding(model_name="sentence-transformers/all-MiniLM-L6-v2")
    # LlamaIndex defaults to OpenAI for LLM, we must disable or mock it to avoid API calls
    Settings.llm = None 
    startup_time = time.time() - startup_start

    # 2. Ingestion (3 docs)
    ingest_start = time.time()
    docs = [
        Document(text="Stratum is a high-density RAG engine built in Rust."),
        Document(text="It uses Candle for local embeddings and redb for ACID storage."),
        Document(text="The engine is designed for AI-native data processing."),
    ]
    index = VectorStoreIndex.from_documents(docs)
    ingestion_time = time.time() - ingest_start

    # 3. Query (Latency)
    query_engine = index.as_query_engine()
    query_start = time.time()
    _response = query_engine.query("What is Stratum?")
    query_latency = time.time() - query_start

    mem = get_mem()
    
    print("--- LlamaIndex Results ---")
    print(f"Startup Time: {startup_time * 1000:.2f} ms")
    print(f"Ingestion Time (3 docs): {ingestion_time * 1000:.2f} ms")
    print(f"Query Latency: {query_latency * 1000:.2f} ms")
    print(f"Memory Usage (RSS): {mem:.2f} MB")
    print(f"Total Execution: {(time.time() - total_start) * 1000:.2f} ms")

if __name__ == "__main__":
    main()
