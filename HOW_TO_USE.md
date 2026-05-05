# How to Use Stratum

Stratum を使用した知識のインジェスト、検索、および精製の手順。

## 1. ライブラリとして使用する

`Cargo.toml` に以下を追加します：

```toml
[dependencies]
stratum = { path = "../stratum" }
```

### 基本的な検索フロー

```rust
use stratum::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. インデックスの初期化
    let mut engine = StratumEngine::new("./storage")?;

    // 2. データのインジェスト (Web/File)
    engine.ingest_url("https://example.com/docs").await?;
    engine.ingest_file("local_knowledge.md").await?;

    // 3. セマンティック検索の実行
    let query = "エージェントの自律性に関する定義は？";
    let results = engine.retrieve(query, 3).await?;

    for node in results {
        println!("Score: {}, Content: {}", node.score, node.content);
    }

    Ok(())
}
```

## 2. CLI ツールとして使用する (開発中)

```bash
# 知識の追加
stratum ingest --file ./docs/architecture.md

# 検索クエリの実行
stratum search "HV-CAD の核心概念は何？"
```

## 3. HV-CAD 統合 (.vlog 連携)

Stratum は検索時に `.vlog` 内の現在の `@CTX` を参照し、動的にスコアリングを調整することが可能です（実装予定）。これにより、単なるキーワード一致を超えた、現在のタスクに最適な「知識の層」を特定します。
