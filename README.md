# Stratum (ストレイタム)

**Stratum** は、Rust で開発された高密度な RAG（検索拡張生成）基盤エンジンです。
知識を地層（Stratum）のように積み上げ、AI エージェントが文脈に応じて必要な情報を瞬時に掘り出し、精製するための「外部記憶」として機能します。

## 概要

Stratum は、単なるベクトルデータベースのラッパーではありません。情報の取り込み（Ingestion）、埋め込み（Embedding）、検索（Retrieval）、および精製（Refining）の全工程を Rust でネイティブ実装した、AI ネイティブな知識処理エンジンです。

## 主な特徴

- **Candle-Powered Local Embedding**:  
  `candle` フレームワークを採用し、外部 API に頼らずバイナリ内で高速なベクトル変換（Embedding）を実行します。
- **High-Density Node Management**:  
  情報は Blake3 ハッシュによる決定論的な ID で管理。メモリ最適化（f16 ベクトル）と高速なシリアライゼーションを両立しています。
- **Trait-Based Modular Ingestion**:  
  トレイトベースの設計により、ファイル、Web、ストリームなど多様なデータソースをシームレスに統合します。
- **HV-CAD Integration**:  
  `lasada` 等の HV-CAD 準拠エージェントと密に連携し、シンボリックな状態（.vlog）に基づいた高度な文脈抽出をサポートします。
- **Zero-Dependency Core**:  
  ランタイム依存を極限まで排除。高速起動とクロスプラットフォームでの安定性を保証します。

## アーキテクチャ

1.  **Refinery (精製レイヤー)**: Raw データをトークナイズし、クリーンなテキスト・ノードへ変換。
2.  **Embedder (埋め込みレイヤー)**: 局所的な AI モデルを使用してベクトル空間へ投影。
3.  **Storage (蓄積レイヤー)**: インデックスとドキュメントを永続化（LanceDB 等の高速ストレージ）。
4.  **Retriever (抽出レイヤー)**: セマンティック検索による最適な情報の特定。

## セットアップ

### 必要条件
- [Rust](https://www.rust-lang.org/) (Cargo, Edition 2021 以上)
- 埋め込みモデル（ONNX/Candle 形式、初回実行時に自動ダウンロード）

### インストール
```bash
git clone https://github.com/kubodad/stratum.git
cd stratum
cargo build --release
```

## 使い方

```rust
// 知識の埋め込みと検索の例
let index = StratumIndex::load("./storage")?;
index.ingest_file("knowledge.txt").await?;

let context = index.retrieve("AIの倫理についての判断基準は？").await?;
println!("Retrieved context: {}", context);
```

## 開発哲学 (System Philosophy)

Stratum は、AI が「知っている」ことと「調べている」ことの境界を滑らかに繋ぐために存在します。
HV-CAD の原則に基づき、知識は揮発的なプロンプトではなく、永続的な「地層（Stratum）」として管理されるべきであるという信念のもと設計されています。

## ライセンス
Apache License 2.0
