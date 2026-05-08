# Stratum: Prerequisites (環境要件)

Stratum をビルドおよび実行するために必要な準備です。

## 1. ツールチェーン
- **Rust**: 1.75 以上 (Edition 2021)
- **Cargo**: Rust 標準のビルド・パッケージ管理ツール

## 2. システムライブラリ (Linux)
ビルドおよび実行に以下のパッケージが必要です。
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev
```

## 3. 埋め込みモデル (Embedding Models)
Stratum はローカル環境で推論を行うため、以下の **ONNX 形式** のモデル資産を事前に用意する必要があります。

- **必要なファイル**:
  - `model.onnx` (モデル本体)
  - `tokenizer.json` (トークナイザー設定)
  - `config.json` (モデル構成)
- **推奨モデル**: `sentence-transformers/all-MiniLM-L6-v2` (ONNX export 版)
- **入手方法**: [Hugging Face](https://huggingface.co/) 等から対象モデルをダウンロードし、ローカルの任意のディレクトリに配置してください。

## 4. 実行環境
- **書き込み権限**: インデックス（`redb` ファイル）を生成・保存するためのディレクトリへの書き込み権限が必要です。
