# mediavault-backend API エンドポイント仕様

**作成日**: 2026-06-22
**関連設計**: [architecture.md](architecture.md)
**関連要件定義**: [requirements.md](../../backend/spec/mediavault-backend/requirements.md)

**【信頼性レベル凡例】**:
- 🔵 **青信号**: EARS要件定義書・受け入れ基準を参考にした確実な定義
- 🟡 **黄信号**: EARS要件定義書・受け入れ基準から妥当な推測による定義
- 🔴 **赤信号**: EARS要件定義書・受け入れ基準にない推測による定義

---

## 共通仕様

### ベースURL 🟡

**信頼性**: 🟡 *既存API仕様なし、tech-stack.mdの構成から妥当な推測*

```
http://localhost:8080/api/v1
```

### 認証 🔵

**信頼性**: 🔵 *REQ-403・NFR-101・TC-018-01/E01/E02より*

内部REST API（`/internal/*`、ファイル登録、インポート等）はAPIキー必須。利用者向けエンドポイントはユーザー認証を持たない（REQ-401、単一ユーザー前提）。

```http
Authorization: Bearer {INTERNAL_API_KEY}
```

未設定・不一致の場合は `401 Unauthorized` を返す。

### エラーレスポンス共通フォーマット 🟡

**信頼性**: 🟡 *既存実装なし、一般的なAPI設計パターンから妥当な推測*

```json
{
  "success": false,
  "error": { "code": "ERROR_CODE", "message": "エラーメッセージ" }
}
```

### ページネーション 🟡

**信頼性**: 🟡 *REQ-001一覧・絞り込みから妥当な推測（具体的パラメータ名はPRD未記載）*

クエリパラメータ: `page`（デフォルト1）, `limit`（デフォルト20、最大100）

```json
{ "success": true, "data": [...], "pagination": { "page": 1, "limit": 20, "total": 100 } }
```

---

## エンドポイント一覧

### アイテム（items）

#### GET /items 🔵

**信頼性**: 🔵 *REQ-001・user-stories 1.4より*

**説明**: コレクション一覧取得（絞り込み対応）

**クエリパラメータ**: `media_type`, `tag_id`, `category_id`, `is_favorite`, `status`, `page`, `limit`（すべてoptional） 🟡 *PRD「一覧・絞り込み」の項目から妥当な推測*

**レスポンス（成功）**: items配列 + pagination

---

#### GET /items/:id 🟡

**信頼性**: 🟡 *REQ-001から妥当な推測（個別取得APIはPRDに直接記載なし）*

**説明**: アイテム詳細取得（メディア別詳細テーブル・タグ・関連付け等を含む）

**エラーコード**: `ITEM_NOT_FOUND`（404）

---

#### POST /items 🔵

**信頼性**: 🔵 *REQ-003・TC-001-01より*

**関連要件**: REQ-003

**説明**: フォーム入力によるアイテム手動作成（`source=manual`）

**リクエスト**:
```json
{ "media_type": "anime", "title": "作品A", "details": {} }
```

**レスポンス（成功, 201）**: 作成済みitem（UUID付き）

**エラーコード**: `VALIDATION_ERROR`（400, media_typeが不正な値等）

---

#### PATCH /items/:id 🔵

**信頼性**: 🔵 *REQ-001・TC-001-02より*

**関連要件**: REQ-001

**説明**: 既存アイテムの部分更新（編集）

**リクエスト**:
```json
{ "rating": 4.5, "is_favorite": true }
```

**レスポンス（成功, 200）**: 更新後item

**エラーコード**: `ITEM_NOT_FOUND`（404, TC-001-E02）

---

#### DELETE /items/:id 🔵

**信頼性**: 🔵 *REQ-001・TC-001-03より*

**関連要件**: REQ-001

**説明**: アイテム削除（関連する item_tags/item_links/item_files等もカスケード削除）

**レスポンス（成功, 204）**

**エラーコード**: `ITEM_NOT_FOUND`（404）

---

#### PATCH /items/:id/status 🔵

**信頼性**: 🔵 *REQ-008・user-stories 1.5より*

**関連要件**: REQ-008

**説明**: 視聴・読了状況（status・consumed_date）の更新

**リクエスト**:
```json
{ "status": "completed", "consumed_date": "2026-06-20" }
```

---

### 外部API検索・インポート

#### GET /items/search 🔵

**信頼性**: 🔵 *REQ-002・TC-002-01/02より*

**関連要件**: REQ-002

**説明**: media_typeに応じた外部API（Jikan/TMDb/NDL/OpenLibrary/Steam/IGDB/AniList）への検索

**クエリパラメータ**: `media_type`（必須）, `q`（必須、検索語）

**レスポンス（成功）**: 外部API検索結果一覧（プロバイダ固有の生データをラップ）

**エラーコード**:
- `API_KEY_NOT_CONFIGURED`（422, TC-002-E01, EDGE-001）
- `EXTERNAL_API_TIMEOUT`（502, TC-002-E02）

---

#### POST /items/import 🔵

**信頼性**: 🔵 *REQ-002・TC-002-03より*

**関連要件**: REQ-002

**説明**: 外部API検索結果からアイテムを新規作成（`source=api`, `external_id`必須）

**リクエスト**: 検索結果の選択データ + media_type

**レスポンス（成功, 201）**: 作成済みitem

---

### グループ・エピソード

#### POST /items/:id/groups 🔵

**信頼性**: 🔵 *REQ-010/011・TC-010-01/TC-011-01より*

**関連要件**: REQ-010, REQ-011, REQ-012, REQ-302

**説明**: シーズン/巻/章グループの作成

**リクエスト**:
```json
{ "group_type": "season", "group_name": "シーズン1", "number": 1, "display_order": 0 }
```

---

#### GET /items/:id/groups 🟡

**信頼性**: 🟡 *REQ-010「一覧表示」から妥当な推測*

**説明**: アイテムに紐づくグループ一覧をdisplay_order順で取得

---

#### POST /groups/:group_id/episodes 🔵

**信頼性**: 🔵 *REQ-010/101・TC-010-01/E01・EDGE-101より*

**関連要件**: REQ-101, EDGE-101

**説明**: 話数登録。`group_type=volume` の場合は拒否される

**リクエスト**:
```json
{ "episode_number": 1, "title": "第1話", "air_date": "2026-01-05" }
```

**エラーコード**: `INVALID_GROUP_TYPE_FOR_EPISODES`（400, TC-010-E01/TC-EDGE-101-01）

---

#### GET /groups/:group_id/episodes 🔵

**信頼性**: 🔵 *REQ-010・TC-010-02より*

**説明**: 話数一覧をepisode_number順に取得

---

### タグ・カテゴリ・マイリスト・関連付け

#### POST /tags ・ DELETE /tags/:id 🔵

**信頼性**: 🔵 *REQ-004・user-stories 2.1より*

#### POST /items/:id/tags/:tag_id ・ DELETE /items/:id/tags/:tag_id 🟡

**信頼性**: 🟡 *REQ-004「付与・削除」から妥当な推測（エンドポイント形式は未記載のため推測）*

#### POST /categories ・ DELETE /categories/:id 🔵

**信頼性**: 🔵 *REQ-004より*

#### POST /mylists ・ POST /mylists/:id/items ・ DELETE /mylists/:id/items/:item_id 🔵

**信頼性**: 🔵 *REQ-005・user-stories 2.2より*

#### POST /item-relations ・ DELETE /item-relations/:id 🔵

**信頼性**: 🔵 *REQ-006/013・TC-013-01/02より*

**リクエスト例**:
```json
{ "item_id": "...", "related_item_id": "...", "relation_type": "dlc" }
```

---

### スタッフ管理

#### POST /staff 🔵

**信頼性**: 🔵 *REQ-009・user-stories 4.1より*

#### POST /items/:id/staff 🔵

**信頼性**: 🔵 *REQ-009より*

**リクエスト**:
```json
{ "staff_id": "...", "role": "監督", "character_name": null }
```

#### DELETE /items/:id/staff/:item_staff_id 🟡

**信頼性**: 🟡 *REQ-009「紐付け」から妥当な推測*

---

### リンク・ファイル・トレーラー

#### POST /items/:id/links ・ DELETE /items/:id/links/:link_id 🔵

**信頼性**: 🔵 *REQ-007・PRD「リンク管理」より*

#### POST /items/:id/files 🔵

**信頼性**: 🔵 *REQ-007/019・TC-007-01より*

**関連要件**: REQ-007, REQ-019

**説明**: ファイルサーバー上の既存パスを指定してファイル登録

**リクエスト**:
```json
{ "path": "/srv/files/pdf/example.pdf", "label": "本編PDF", "file_type": "pdf" }
```

---

#### POST /items/:id/files/upload 🔵

**信頼性**: 🔵 *REQ-019/104・TC-019-01/E01より*

**関連要件**: REQ-019, REQ-104, EDGE-003

**説明**: バイナリ直接アップロード。`file_type`に応じて `/srv/files/pdf` または `/srv/media/photos` に配置し、相対パスを保存。書込失敗時は`item_files`レコードを作成しない

**リクエスト**: `multipart/form-data`（file, file_type, label）

**エラーコード**: `FILE_STORAGE_WRITE_FAILED`（500, TC-019-E01）

---

#### PATCH /items/:id/files/:file_id/calibre-link 🔵

**信頼性**: 🔵 *REQ-020/103・TC-020-01より*

**関連要件**: REQ-020, REQ-103

**説明**: PDFファイルに`calibre_book_id`を紐付け更新（Calibre-Web取込完了後）

**リクエスト**:
```json
{ "calibre_book_id": "calibre-12345" }
```

---

#### POST /items/:id/trailers ・ DELETE /items/:id/trailers/:trailer_id 🔵

**信頼性**: 🔵 *REQ-007・PRD「トレーラー管理」より*

---

### インポート

#### POST /import/booklog 🔵

**信頼性**: 🔵 *REQ-016・TC-016-01/E01より*

**関連要件**: REQ-016, EDGE-002

**説明**: ブクログCSV（multipart）から一括インポート。不正行はスキップし理由を記録

**レスポンス（成功, 200）**: `ImportSummary`（success_count, failure_count, failures）

---

#### POST /import/steam 🔵

**信頼性**: 🔵 *REQ-017・TC-017-01/E01より*

**関連要件**: REQ-017, EDGE-002

**説明**: Steam Web API（steam_id指定）からライブラリ一括インポート

**リクエスト**:
```json
{ "steam_id": "76561198000000000" }
```

**エラーコード**: `STEAM_API_KEY_INVALID`（401, TC-017-E01）

---

### 外部APIキー管理

#### PUT /settings/api-keys/:provider 🔵

**信頼性**: 🔵 *REQ-015/NFR-202・TC-015-01/02/03より*

**関連要件**: REQ-015, NFR-202

**説明**: 外部APIキー（tmdb/igdb/ndl/steam/openlibrary/anilist）の登録・更新

**リクエスト**:
```json
{ "api_key": "xxxxx" }
```

**エラーコード**: `INVALID_PROVIDER`（400, TC-015-02）

---

### 内部REST API（巡回バッチ・ファイルサーバー監視向け） 🔵

**信頼性**: 🔵 *REQ-018・user-stories 6.1・TC-018-01/E01/E02より*

すべて `Authorization: Bearer {INTERNAL_API_KEY}` 必須。

#### POST /internal/items 🔵

**説明**: アイテム新規登録（手動 or 外部API取得結果のインポート、`/items`・`/items/import`と同等処理を内部APIキー認証で提供）

#### PATCH /internal/items/:id 🔵

**説明**: 既存アイテムのメタデータ部分更新

#### GET /internal/items/search 🔵

**説明**: タイトル・media_type・タグ・外部IDなどでアイテムを条件検索

#### POST /internal/items/:id/groups ・ POST /internal/groups/:group_id/episodes 🔵

**説明**: グループ/エピソードの登録・更新（巡回バッチからの話数同期用）

#### POST /internal/items/:id/files 🔵

**説明**: ファイルサーバー上のパスを紐付け登録（監視プロセスからの新規ファイル検知時に使用）

---

## レート制限 🔴

**信頼性**: 🔴 *PRD・要件定義に記載なし、単一ユーザー・セルフホスト前提のため不要と判断*

単一ユーザー運用のため、レート制限は導入しない。

## バージョニング 🟡

**信頼性**: 🟡 *既存API仕様なし、一般的な慣習から妥当な推測*

APIバージョンはURLパスに含める（`/api/v1/`）。内部APIは `/internal/` 配下とし、バージョンプレフィックスは付与しない。

## CORS設定 🟡

**信頼性**: 🟡 *セルフホスト・フロントエンド分離構成から妥当な推測*

フロントエンドのオリジンのみ許可（具体的なオリジンは環境変数で設定）。

## 関連文書

- **アーキテクチャ**: [architecture.md](architecture.md)
- **型定義**: [types.rs](types.rs)
- **データフロー**: [dataflow.md](dataflow.md)
- **要件定義**: [requirements.md](../../backend/spec/mediavault-backend/requirements.md)

## 信頼性レベルサマリー

- 🔵 青信号: 32件 (68%)
- 🟡 黄信号: 14件 (30%)
- 🔴 赤信号: 1件 (2%)（レート制限非導入の判断）

**品質評価**: 高品質（エンドポイント形式の一部はPRD未記載のため🟡推測だが、要件・受け入れ基準との対応は明確）
