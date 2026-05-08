import subprocess
import venv
import os
import sys

def setup():
    env_dir = "venv_benchmark"
    if not os.path.exists(env_dir):
        print(f"Creating virtual environment in {env_dir}...")
        venv.create(env_dir, with_pip=True)
    
    pip_exe = os.path.join(env_dir, "bin", "pip")
    print("Installing llama-index, psutil, sentence-transformers, torch...")
    subprocess.check_call([pip_exe, "install", "--quiet", "llama-index", "llama-index-embeddings-huggingface", "psutil", "torch"])
    print("Environment ready.")

if __name__ == "__main__":
    setup()
