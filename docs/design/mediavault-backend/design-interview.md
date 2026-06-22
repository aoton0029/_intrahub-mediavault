# mediavault-backend 設計ヒアリング記録

**作成日**: 2026-06-22
**ヒアリング実施**: step2〜step4

## ヒアリング目的

要件定義（requirements.md / user-stories.md / acceptance-criteria.md / interview-record.md）と既存コードベース（`backend/src`, `backend/api-client-lib`）の調査結果を踏まえ、設計フェーズで確定すべき作業規模とコード分析範囲を確認した。

## 質問と回答

### Q1: 設計の作業規模

**質問日時**: 2026-06-22
**カテゴリ**: 優先順位
**背景**: フル設計・軽量設計・カスタムのいずれで進めるかを確定する必要があった。

**回答**: フル設計（推奨）を選択。

**信頼性への影響**: architecture.md・dataflow.mdに加え、types.rs（型定義）・database-schema.sql・api-endpoints.mdを含む全ファイルを🔵基調で作成する方針を確定。

---

### Q2: 既存実装（api-client-lib）の詳細コード分析の必要性

**質問日時**: 2026-06-22
**カテゴリ**: 既存設計確認
**背景**: ヒアリング記録（interview-record.md Q1）で「backend/srcは完全スケルトン、api-client-libは実装済み」と既に判明していたため、追加の網羅的調査が必要か確認した。

**回答**: 不要（推奨）を選択。設計時に `traits.rs`（`ApiClient`トレイト定義）と `lib.rs`（公開モジュール一覧）のみ確認すれば十分と判断。

**信頼性への影響**: `ApiClient::execute(Request) -> Result<ApiResponse<Model>, ApiError>` という統一インターフェースの存在を🔵で確認済みとし、architecture.mdの外部APIクライアント節に反映。各プロバイダ別の内部実装（jikan.rs等の詳細ロジック）は設計対象外とし、API本体側からは既存traitを呼び出すラッパーとして扱う方針とした。

## ヒアリング結果サマリー

### 確認できた事項
- 設計規模はフル設計
- `backend/src` は空スケルトン、ゼロから `mediavault-api` クレートを実装する
- `api-client-lib` は7プロバイダ分のクライアントが実装済みで、`ApiClient`トレイトを介して統一的に呼び出し可能
- 既存の `docs/design/` ディレクトリ・`docs/rule/` ディレクトリは存在せず、本設計が最初の設計文書となる

### 設計方針の決定事項
- レイヤードアーキテクチャ（routes/handlers/services/db/middleware/models）を採用
- 外部API呼び分けは `services::ExternalSearchService` 等の新規ラッパーで `api-client-lib` を利用する
- DBスキーマは `items` 共通テーブル + メディア別詳細テーブルのJOIN構成（REQ-405準拠）

### 残課題
- 現在の `backend/Cargo.toml` のworkspace実構成（`mediavault-api`クレートが未作成かどうか）は本設計時点では未確認のまま、tech-stack.md記載の構成を正とした 🟡
- ブクログCSVの具体的カラムフォーマットは実装着手前に実物サンプルでの確認が必要（interview-record.md記載の残課題を継承）

### 信頼性レベル分布

**ヒアリング前**（要件定義書群のみからの素案）:
- 🔵 青信号: 約28件
- 🟡 黄信号: 約9件
- 🔴 赤信号: 0件

**ヒアリング後**（本設計ヒアリング反映後、全設計文書通算）:
- 🔵 青信号: 約60件
- 🟡 黄信号: 約20件
- 🔴 赤信号: 0件

## 関連文書

- **アーキテクチャ設計**: [architecture.md](architecture.md)
- **データフロー**: [dataflow.md](dataflow.md)
- **型定義**: [types.rs](types.rs)
- **DBスキーマ**: [database-schema.sql](database-schema.sql)
- **API仕様**: [api-endpoints.md](api-endpoints.md)
- **要件定義**: [requirements.md](../../backend/spec/mediavault-backend/requirements.md)
