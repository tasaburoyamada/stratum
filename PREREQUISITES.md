# Stratum: Prerequisites

Stratum をビルドおよび実行するために必要な環境要件。

## 1. 開発環境
- **Rust**: Edition 2021 以上を推奨。
  - `rustup` を使用して最新の stable チャンネルを使用してください。
- **Cargo**: ビルドおよび依存関係管理に使用します。

## 2. 依存ライブラリ
Stratum は可能な限り Rust ネイティブな依存関係を採用していますが、以下のツールが必要になる場合があります。

### OS パッケージ (Ubuntu/WSL2)
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev
```

### Tensor 演算の加速 (オプション)
- **CUDA**: NVIDIA GPU を使用して Embedding を加速する場合、CUDA Toolkit 11.8 以上が必要です。
- **MKL/OpenBLAS**: CPU での演算を最適化する場合に使用します。

## 3. モデル資材
初回実行時に、以下のモデル（HuggingFace より）が自動的にダウンロードされ、`~/.cache/stratum` 等に配置されます。
- `sentence-transformers/all-MiniLM-L6-v2` (デフォルトの埋め込みモデル)
- トークナイザー設定ファイル (`tokenizer.json`)
