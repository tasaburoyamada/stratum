# How to Use Stratum

Stratum をライブラリとして使用し、ドキュメントのインジェスト（取り込み）と検索を行うための基本手順です。

## 1. ライブラリの導入

`Cargo.toml` に以下を追加します。

```toml
[dependencies]
stratum = { path = "path/to/stratum" }
tokio = { version = "1.0", features = ["full"] }
```

## 2. 基本的な RAG フロー

以下の 3 ステップで、ローカルデータに対するセマンティック検索を実装できます。

### ステップ 1: モデルとストレージの準備
`CandleEmbedding` を初期化し、`StorageContext` でデータの保存先を指定します。

```rust
use std::sync::Arc;
use stratum::embeddings::candle::CandleEmbedding;
use stratum::storage::storage_context::StorageContext;

// ローカルの ONNX モデルをロード
let embed_model = Arc::new(CandleEmbedding::new(
    "path/to/model.onnx",
    "path/to/tokenizer.json",
    "path/to/config.json",
    None
)?);

// 【永続化モード】 redb (ACID準拠) によるストレージ管理
let storage_context_persistent = StorageContext::from_dir("./my_storage")?;

// 【超高速インメモリモード】 一時的な処理やバッチ検索用
let storage_context_memory = StorageContext::in_memory();
```

### ステップ 2: データの取り込みと分割
ディレクトリ内のファイルを読み込み、検索に適したサイズ（ノード）に分割します。

```rust
use stratum::readers::file::SimpleDirectoryReader;
use stratum::node_parser::sentence_splitter::SentenceSplitter;

// 1. ファイルの読み込み
let reader = SimpleDirectoryReader::new("path/to/data", true, Some(vec!["md".into()]), None);
let documents = reader.load_data().await?;

// 2. テキストの分割 (Node化)
let nodes = SentenceSplitter::default().transform(documents).await?;
```

### ステップ 3: インデックス構築と検索
ベクトルインデックスを構築し、クエリに対して最も関連性の高い情報を抽出します。

```rust
use stratum::indices::vector_store::VectorStoreIndex;
use stratum::retrievers::vector_store_retriever::VectorIndexRetriever;
use stratum::core::query_bundle::QueryBundle;

// インデックスの構築
let index = Arc::new(VectorStoreIndex::from_nodes(
    nodes,
    storage_context,
    embed_model,
    Default::default()
).await?);

// 検索の実行 (上位3件を取得)
let retriever = VectorIndexRetriever::new(index, 3, None);
let results = retriever.retrieve(QueryBundle::new("Rustの特徴は？")).await?;

for res in results {
    println!("Score: {:?}, Content: {:?}", res.score, res.node.content);
}
```

## 3. Python バインディング (PyO3) の利用

Python 環境から Stratum を呼び出す極薄ラッパーを提供しています。これにより Python スクリプトから Rust の超高速・省メモリなコアエンジンを透過的に利用可能です。

### インストール手順

Python プロジェクトの仮想環境内で以下を実行します。

```bash
# Maturinのインストール
pip install maturin

# Stratum の Python バインディングディレクトリへ移動
cd bindings/python

# ビルドして現在の仮想環境へインストール (Rust側のコンパイルが走ります)
maturin develop --release
```

### Python からの利用例

```python
import stratum_rag

# In-Memory モードで初期化 (永続化する場合はディレクトリパスを渡す)
engine = stratum_rag.Stratum()

# 稼働確認
print(engine.ping())
# => "Stratum Core (Rust) is active."
```

*※ 現在 Python 側のインターフェースは意図的に「極薄」に保たれています。高度なデータ制御やカスタマイズは Rust 側 (`src/`) で完結させることが本プロジェクトの設計思想です。*

