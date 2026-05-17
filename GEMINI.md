# Stratum Knowledge Engine: HV-CAD Knowledge Stratification

本プロジェクトは、AIエージェントの長期記憶と知識精査を司る、HV-CAD準拠の決定論的知識エンジンである。
情報の「地層化（Stratification）」と「精錬（Refining）」を通じて、単なる検索を超えた高密度な文脈提供を実現する。

## 1. 根本原則 (Core Principles)
- **Knowledge Stratification**: 情報はフラットなリストではなく、時間（Temporal）、信頼度（Confidence）、重要度（Importance）に基づく「地層」として管理されるべきである。
- **Deterministic Representation**: AI（LLM/Embedding）が最も効率的に消化できる形式で保持しつつ、Blake3ハッシュによる一貫性と再現性を保証する。
- **Zero-Copy Pipeline**: ネットワークからベクトル空間まで、ディスクI/Oと中間変換を最小化したストリーミング処理を徹底する。
- **HV-CAD Alignment**: 検索プロセスは動的な `.vlog` 状態（@CTX, @BIAS）に同期し、エージェントの現在の意図を反映する。

## 2. 実装指針 (Implementation Guidelines)
- **Layered Retrieval**: 検索は単一のベクトル類似度だけでなく、地層（層状のフィルタリング）を考慮した多段評価を行う。
- **Symbolic Control**: 知識の精製プロセスはシンボリックに定義され、人間が介入可能な評価関数によって正則化される。
- **Type-Safe Ingestion**: Rustの型システムによりデータの整合性と変換プロセスをコンパイル時に保証する。

## 3. 禁止事項 (Prohibitions)
- Python版LlamaIndexのクラス構造の無批判な移植。
- 確率論的な「何となくの検索」に依存し、地層化プロセスを怠ること。
- 人間向けの中間生成物を強制的に書き出すこと。
