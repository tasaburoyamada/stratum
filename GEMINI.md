# LlamaIndex Rust Implementation: Stochastic Data Engine

本プロジェクトは、Python版LlamaIndexの冗長な抽象化と不潔な依存関係を排し、Rustによる「AIネイティブなデータ精錬所」を構築することを目的とする。

## 1. 根本原則 (Core Principles)
- **AI-First Representation**: データは人間が読むための形式ではなく、AI（LLM/Embedding）が最も効率的に消化できるテンソル/バイナリ形式で保持する。
- **Zero-Copy Pipeline**: ネットワークからベクトル空間まで、ディスクI/Oと人間向けインターフェースを最小化したストリーミング処理を徹底する。
- **Type-Safe Ingestion**: 動的な辞書渡しを排し、Rustの型システムによってデータの整合性と変換プロセスをコンパイル時に保証する。

## 2. 実装指針 (Implementation Guidelines)
- **Minimal Interface**: 人間とのインターフェースはボトルネックである。設定、ログ、操作は極力シンボリックに行い、自然言語による介在を減らす。
- **Modular Refiner**: 各データソース（URL, AST, etc.）は独立した「精錬所（Refiner）」として実装し、オンメモリでベクトル化を行う。
- **Durable State**: 状態管理は `.vlog` 形式で行い、HV-CADフレームワークとの統合を維持する。

## 3. 禁止事項 (Prohibitions)
- Python版のクラス構造の無批判な移植（「ズレた」実装の再生産）。
- 中間生成物としての人間向けファイルの強制的な書き出し。
