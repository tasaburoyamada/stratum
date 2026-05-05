# Project: Stratum (Rust / HV-CAD Knowledge Engine)

高密度な RAG（検索拡張生成）の実装を通じて、AI エージェントの長期記憶と知識精製を司る、HV-CAD 準拠の基盤プロジェクト。

## 1. 目的
- **ローカル完結型の知識埋め込み**: `candle` を用いた内蔵型 Embedding により、プライバシーとパフォーマンスを両立し、外部 API への依存を排除する。
- **決定論的な情報管理**: Blake3 ハッシュによるノード ID 生成とシンボリックな状態管理により、情報の永続性と再現性を確保する。
- **HV-CAD エージェントとの融合**: `lasada` 等の実行エンジンに対し、単なる文字列検索ではなく、現在の .vlog 状態に同期した「文脈の層（Stratum）」を提供する。

## 2. 建築原則 (Architectural Principles)
- **地層化 (Stratification)**: 知識はフラットなリストではなく、時間、信頼度、および重要度によって層状に積み重なるべきである。
- **ストリーム精製 (Stream Refining)**: インジェストされた Raw データは、リアルタイムでトークナイズおよびノード化され、検索可能な「精製済み知識」へと変換される。
- **シンボリック検索**: 自然言語の曖昧なクエリを、HV-CAD のコンテキスト（@CTX）に基づいたベクトル座標へ変換し、最も関連性の高い情報の地層を特定する。

## 3. 技術目標 (Technical Goals)
- **Candle 統合**: Rust ネイティブな深層学習フレームワークによる Embedding パイプラインの構築。
- **高性能ストレージ**: LanceDB (Apache Arrow) 等を採用し、大規模な知識ベースに対してもサブミリ秒の検索レスポンスを実現する。
- **マルチソース・インジェスタ**: ファイルシステム、Web（Crawler）、および JIT なストリームデータに対応するモジュール設計。

## 4. ロードマップ (Phase)
### Phase 1: コア・インジェスト設計 [DONE]
- [x] `Node` 構造体と Blake3 ベースの ID 生成の実装
- [x] トレイトベースの Reader インターフェースの定義

### Phase 2: ローカル Embedding 実装 [DONE]
- [x] `candle-transformers` による BERT/MiniLM 等のモデル実行環境の構築
- [x] インメモリ・ベクトルストアによるセマンティック検索の検証

### Phase 3: 永続化 & HV-CAD 統合
- [ ] LanceDB 等による永続化ストレージの実装
- [ ] `.vlog` からの動的な検索フィルタリング機能の追加
- [ ] `lasada` 向け JIT コンテキスト注入 API の提供

## 5. 技術スタック
- **言語**: Rust
- **Embedding**: Candle (HuggingFace)
- **状態管理**: HV-CAD (.vlog)
- **シリアライゼーション**: Serde, Bincode
- **ライセンス**: Apache License 2.0
