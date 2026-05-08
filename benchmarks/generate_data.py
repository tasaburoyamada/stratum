import os

def generate():
    os.makedirs("benchmarks/data_large", exist_ok=True)
    template = """# Document {i}
    
Stratum is a high-density RAG engine built in Rust. It uses Candle for local embeddings and redb for ACID storage.
The engine is designed for AI-native data processing, ensuring memory safety and extreme performance.
This is document number {i}. It contains varied information about system architecture.
In chapter {i}, we explore the fundamental limits of computation and memory bandwidth.
Rust's zero-cost abstractions allow us to build complex hierarchies without sacrificing speed.
The integration with Python via PyO3 provides a seamless experience for data scientists while maintaining the robust core.
"""
    for i in range(1, 101): # 100 documents
        with open(f"benchmarks/data_large/doc_{i}.md", "w") as f:
            f.write(template.format(i=i))
            
if __name__ == "__main__":
    generate()
