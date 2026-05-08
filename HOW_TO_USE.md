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

// redb (ACID準拠) によるストレージ管理
let storage_context = StorageContext::from_dir("./my_storage")?;
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
