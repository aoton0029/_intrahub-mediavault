# mediavault-backend タスク概要

**作成日**: 2026-06-22
**プロジェクト期間**: 約21日（164時間 ÷ 1日8時間換算）
**推定工数**: 164時間
**総タスク数**: 34件

## 関連文書

- **要件定義書**: [📋 requirements.md](../../spec/mediavault-backend/requirements.md)
- **ユーザストーリー**: [📖 user-stories.md](../../spec/mediavault-backend/user-stories.md)
- **受け入れ基準**: [✅ acceptance-criteria.md](../../spec/mediavault-backend/acceptance-criteria.md)
- **準備タスク**: [🔧 prep.md](../../spec/mediavault-backend/prep.md)
- **設計文書**: [📐 architecture.md](../../design/mediavault-backend/architecture.md)
- **API仕様**: [🔌 api-endpoints.md](../../design/mediavault-backend/api-endpoints.md)
- **データベース設計**: [🗄️ database-schema.sql](../../design/mediavault-backend/database-schema.sql)
- **型定義**: [📝 types.rs](../../design/mediavault-backend/types.rs)
- **データフロー図**: [🔄 dataflow.md](../../design/mediavault-backend/dataflow.md)

## フェーズ構成

| フェーズ | 目標 | タスク数 | 工数 | ファイル |
|---------|------|----------|------|----------|
| Phase 1 | 基盤構築（Cargo workspace・Docker・マイグレーション・共通基盤） | 7 | 31h | [TASK-0001~0007](#phase-1-基盤構築) |
| Phase 2 | コアCRUD実装（items・タグ/カテゴリ/マイリスト/関連付け/グループ/スタッフ/リンク） | 14 | 68h | [TASK-0008~0021](#phase-2-コアcrud実装) |
| Phase 3 | 外部API連携（検索・インポート・APIキー管理） | 4 | 21h | [TASK-0022~0025](#phase-3-外部api連携) |
| Phase 4 | ファイル管理・拡張機能（パス登録・アップロード・Calibre連携） | 3 | 13h | [TASK-0026~0028](#phase-4-ファイル管理拡張機能) |
| Phase 5 | 内部API・インポート（巡回バッチ向けAPI・ブクログ/Steamインポート） | 3 | 18h | [TASK-0029~0031](#phase-5-内部apiインポート) |
| Phase 6 | 統合テスト・仕上げ（E2E・CI・README） | 3 | 13h | [TASK-0032~0034](#phase-6-統合テスト仕上げ) |

## タスク番号管理

**使用済みタスク番号**: TASK-0001 ~ TASK-0034
**次回開始番号**: TASK-0035

## 全体進捗

- [ ] Phase 1: 基盤構築
- [ ] Phase 2: コアCRUD実装
- [ ] Phase 3: 外部API連携
- [ ] Phase 4: ファイル管理・拡張機能
- [ ] Phase 5: 内部API・インポート
- [ ] Phase 6: 統合テスト・仕上げ

## マイルストーン

- **M1: 基盤完成**: Cargo workspace・Docker・DBマイグレーション・APIキー検証ミドルウェア・Axum起動骨格が完了（Phase 1終了時）
- **M2: コアCRUD完成**: items共通CRUD・タグ/カテゴリ/マイリスト/関連付け/グループ・エピソード/スタッフ/リンクAPIが完了（Phase 2終了時）
- **M3: 外部連携・拡張完成**: 外部API検索・インポート・ファイル管理（パス/アップロード/Calibre連携）が完了（Phase 4終了時）
- **M4: リリース準備完了**: 内部API・ブクログ/Steamインポート・統合テスト・CI・READMEが完了（Phase 6終了時）

---

## Phase 1: 基盤構築

**目標**: Rust/Axum APIサーバーの実行基盤を整備する
**成果物**: `mediavault-api`クレート、docker-compose.yml、DBマイグレーション一式、共通エラー型、APIキー検証ミドルウェア、起動可能なAxumサーバー

### タスク一覧

- [x] [TASK-0001: Cargo workspaceへのmediavault-apiクレート追加と依存パッケージ設定](TASK-0001.md) - 4h (DIRECT) 🔵
- [x] [TASK-0002: docker-compose.yml作成（Postgresコンテナ＋アプリコンテナ）](TASK-0002.md) - 3h (DIRECT) 🔵 ✅完了 (2026-06-23)
- [x] [TASK-0003: sqlx-cli導入と初期マイグレーション作成（ENUM型＋items＋詳細テーブル）](TASK-0003.md) - 6h (DIRECT) 🔵 ✅完了 (2026-06-23)
- [x] [TASK-0004: 残りのマイグレーション作成（関連テーブル群・トリガー）](TASK-0004.md) - 6h (DIRECT) 🔵 ✅完了 (2026-06-23)
- [x] [TASK-0005: 共通エラー型・統一APIレスポンス実装](TASK-0005.md) - 4h (TDD) 🟡 ✅完了 (2026-06-23)
- [x] [TASK-0006: 内部API用APIキー検証ミドルウェア実装](TASK-0006.md) - 4h (TDD) 🔵 ✅完了 (2026-06-23)
- [x] [TASK-0007: Axumルーター骨格・DB接続プール設定・main.rs実装](TASK-0007.md) - 4h (DIRECT) 🔵 ✅完了 (2026-06-23)

### 依存関係

```
TASK-0001 → TASK-0002 → TASK-0003 → TASK-0004 → Phase2全タスク
TASK-0001 → TASK-0005
TASK-0001 → TASK-0006
TASK-0001, TASK-0003 → TASK-0007 → Phase2全タスク
```

---

## Phase 2: コアCRUD実装

**目標**: items共通テーブルとその関連エンティティのCRUD APIを実装する
**成果物**: items CRUD、タグ/カテゴリ、マイリスト、関連付け（reference/dlc）、グループ・エピソード、スタッフ、リンク/トレーラーAPI

### タスク一覧

- [x] [TASK-0008: itemsモデル・リクエストDTO・バリデーション実装](TASK-0008.md) - 6h (TDD) 🔵 ✅完了 (2026-06-23)
- [x] [TASK-0009: POST /items（手動作成）実装](TASK-0009.md) - 4h (TDD) 🔵 ✅完了 (2026-06-23)
- [x] [TASK-0010: GET /items一覧・絞り込み実装](TASK-0010.md) - 6h (TDD) 🔵 ✅完了 (2026-06-23)
- [x] [TASK-0011: GET /items/:id 詳細取得実装](TASK-0011.md) - 4h (TDD) 🟡 ✅完了 (2026-06-23)
- [ ] [TASK-0012: PATCH /items/:id 部分更新実装](TASK-0012.md) - 4h (TDD) 🔵
- [ ] [TASK-0013: DELETE /items/:id 実装（カスケード削除）](TASK-0013.md) - 4h (TDD) 🔵
- [ ] [TASK-0014: PATCH /items/:id/status 実装](TASK-0014.md) - 3h (TDD) 🔵
- [ ] [TASK-0015: タグ・カテゴリCRUD実装](TASK-0015.md) - 6h (TDD) 🔵
- [ ] [TASK-0016: マイリストCRUD実装](TASK-0016.md) - 6h (TDD) 🔵
- [ ] [TASK-0017: item_relations（関連付け・DLC）CRUD実装](TASK-0017.md) - 5h (TDD) 🔵
- [ ] [TASK-0018: item_groups（シーズン/巻/章）CRUD実装](TASK-0018.md) - 5h (TDD) 🔵
- [ ] [TASK-0019: item_episodes CRUD + EDGE-101検証実装](TASK-0019.md) - 6h (TDD) 🔵
- [ ] [TASK-0020: スタッフ管理CRUD実装](TASK-0020.md) - 5h (TDD) 🔵
- [ ] [TASK-0021: item_links/item_trailers CRUD実装](TASK-0021.md) - 4h (TDD) 🔵

### 依存関係

```
TASK-0004, TASK-0007 → TASK-0008 → TASK-0009, TASK-0010, TASK-0011, TASK-0012, TASK-0013, TASK-0014
TASK-0004, TASK-0007 → TASK-0015
TASK-0004, TASK-0007 → TASK-0016
TASK-0004, TASK-0007 → TASK-0017
TASK-0004, TASK-0007 → TASK-0018 → TASK-0019
TASK-0004, TASK-0007 → TASK-0020
TASK-0004, TASK-0007 → TASK-0021
```

---

## Phase 3: 外部API連携

**目標**: `api-client-lib`を活用した外部API検索・インポート機能を実装する
**成果物**: 外部APIキー管理、ExternalSearchServiceラッパー、検索API、インポートAPI

### タスク一覧

- [ ] [TASK-0022: api_credentials（外部APIキー管理）CRUD実装](TASK-0022.md) - 5h (TDD) 🔵
- [ ] [TASK-0023: ExternalSearchServiceラッパー実装（media_type→provider振り分け）](TASK-0023.md) - 6h (TDD) 🔵
- [ ] [TASK-0024: GET /items/search 実装](TASK-0024.md) - 5h (TDD) 🔵
- [ ] [TASK-0025: POST /items/import 実装](TASK-0025.md) - 5h (TDD) 🔵

### 依存関係

```
TASK-0004, TASK-0007 → TASK-0022 → TASK-0023 → TASK-0024 → TASK-0025
TASK-0009（itemsトランザクション処理） → TASK-0025
```

---

## Phase 4: ファイル管理・拡張機能

**目標**: ファイルサーバー連携（パス登録・バイナリアップロード・Calibre-Web連携）を実装する
**成果物**: item_filesのパス指定方式・バイナリアップロード方式、calibre_book_id紐付け

### タスク一覧

- [ ] [TASK-0026: POST /items/:id/files（パス指定方式）実装](TASK-0026.md) - 4h (TDD) 🔵
- [ ] [TASK-0027: POST /items/:id/files/upload（バイナリ直接アップロード）実装](TASK-0027.md) - 6h (TDD) 🔵
- [ ] [TASK-0028: PATCH /items/:id/files/:file_id/calibre-link 実装](TASK-0028.md) - 3h (TDD) 🔵

### 依存関係

```
TASK-0008 → TASK-0026 → TASK-0027
TASK-0026 → TASK-0028
```

---

## Phase 5: 内部API・インポート

**目標**: 巡回バッチ・ファイルサーバー監視プロセス向けの内部REST APIと、ブクログ/Steam一括インポートを実装する
**成果物**: `/internal/*`エンドポイント群、ブクログCSVインポート、Steamライブラリインポート

### タスク一覧

- [ ] [TASK-0029: 内部REST APIルート群実装（/internal/items等）](TASK-0029.md) - 6h (TDD) 🔵
- [ ] [TASK-0030: ブクログCSVインポート実装](TASK-0030.md) - 6h (TDD) 🟡
- [ ] [TASK-0031: Steamライブラリインポート実装](TASK-0031.md) - 6h (TDD) 🔵

### 依存関係

```
TASK-0006, TASK-0009, TASK-0010, TASK-0011, TASK-0012, TASK-0013, TASK-0018, TASK-0019, TASK-0026 → TASK-0029
TASK-0009 → TASK-0030
TASK-0023, TASK-0009 → TASK-0031
```

---

## Phase 6: 統合テスト・仕上げ

**目標**: 全機能の統合テスト・CI整備・ドキュメント整備を行う
**成果物**: E2E統合テスト、GitHub Actions CI設定、README

### タスク一覧

- [ ] [TASK-0032: 主要フロー統合テスト実装](TASK-0032.md) - 6h (TDD) 🔵
- [ ] [TASK-0033: CI設定（GitHub Actions）](TASK-0033.md) - 4h (DIRECT) 🔵
- [ ] [TASK-0034: README・起動手順整備](TASK-0034.md) - 3h (DIRECT) 🟡

### 依存関係

```
TASK-0001〜0031（全て） → TASK-0032 → TASK-0033 → TASK-0034
```

---

## 信頼性レベルサマリー

### 全タスク統計（タスク単位の代表信頼性レベル）

- **総タスク数**: 34件
- 🔵 **青信号**: 30件 (88%)
- 🟡 **黄信号**: 4件 (12%)
- 🔴 **赤信号**: 0件 (0%)

### フェーズ別信頼性

| フェーズ | 🔵 青 | 🟡 黄 | 🔴 赤 | 合計 |
|---------|-------|-------|-------|------|
| Phase 1 | 6 | 1 | 0 | 7 |
| Phase 2 | 13 | 1 | 0 | 14 |
| Phase 3 | 4 | 0 | 0 | 4 |
| Phase 4 | 3 | 0 | 0 | 3 |
| Phase 5 | 2 | 1 | 0 | 3 |
| Phase 6 | 2 | 1 | 0 | 3 |

**🟡となった4タスクの理由**:
- TASK-0005（共通エラー型）: エラーコード一覧の網羅性が設計文書に明示なく妥当な推測
- TASK-0011（GET /items/:id）: 個別取得APIの詳細仕様がPRD/受け入れ基準に直接記載なし
- TASK-0030（ブクログCSVインポート）: prep.mdに「実カラム形式は実物サンプル確認が必要」と明記されている残課題のため
- TASK-0034（README整備）: 既存実装パターンからの妥当な推測

**品質評価**: 高品質（赤信号なし。黄信号は実装着手前にサンプル確認や詳細仕様確定が必要な項目のみ）

## クリティカルパス

```
TASK-0001 → TASK-0002 → TASK-0003 → TASK-0004 → TASK-0008 → TASK-0009
→ TASK-0023(依存: TASK-0022) → TASK-0024 → TASK-0025
→ TASK-0029(依存: TASK-0006,0009-0013,0018,0019,0026) → TASK-0032 → TASK-0033 → TASK-0034
```

**クリティカルパス工数**: 約110時間（基盤構築→items基本CRUD→外部API連携→内部API→統合テスト・仕上げの直列部分）
**並行作業可能工数**: 約54時間（タグ/カテゴリ・マイリスト・関連付け・スタッフ・リンク等、items基盤完成後に並行実装可能なタスク群）

## 次のステップ

タスクを実装するには:
- 全タスク順番に実装: `/tsumiki:kairo-implement`
- 特定タスクを実装: `/tsumiki:kairo-implement TASK-0001`
