# TDDテストケース定義書: GET /items（一覧・絞り込み）

- **機能名**: GET /items（一覧・絞り込み）
- **タスクID**: TASK-0010
- **要件名**: mediavault-backend
- **フェーズ**: Phase 2 - コアCRUD実装
- **作成日**: 2026-06-23
- **前フェーズ成果物**: `docs/implements/mediavault-backend/TASK-0010/get-items-list-requirements.md`, `docs/implements/mediavault-backend/TASK-0010/note.md`

**【信頼性レベル凡例】**:
- 🔵 **青信号**: 要件定義書・設計文書・既存実装を参考にしてほぼ推測していない場合
- 🟡 **黄信号**: 要件定義書・設計文書から妥当な推測の場合
- 🔴 **赤信号**: 要件定義書・設計文書にない推測の場合

---

## 0. 設計上の事前確定事項（要件定義の未確定3項目の解決）

要件定義書 6章「未確定事項」を、本テストケース定義フェーズで以下の通り**確定**する。テストケースはこの確定方針を前提とする。

### 確定1: ページネーション付き一覧レスポンス型 🟡
- 既存 `ApiOk<T>`（`{ success, data }`）は `pagination` を持たないため、`models/response.rs` に**新規ラッパー型 `PaginatedOk<T>`** を追加する。
  - 形: `{ "success": true, "data": T, "pagination": { "page", "limit", "total" } }`
  - `pagination` 部は `Pagination { page: u32, limit: u32, total: i64 }`（`total` は `COUNT(*)` の `i64` をそのまま保持）。
  - `ApiOk<T>` 同様に `#[derive(Serialize)]` + `PaginatedOk::new(data, pagination)` コンストラクタ + `IntoResponse`（200 OK 固定）を実装する。
  - *根拠*: 要件定義 2.2 補足（🟡）、`models/response.rs` の `ApiOk<T>` 既存規約。

### 確定2: page=0 / limit=0 等の下限境界はクランプ（エラーにしない） 🟡
- `page < 1 → 1`、`limit < 1 → 20`（デフォルト）、`limit > 100 → 100` に**クランプ**する（不正値でも 400 にはせず 200 で正常応答）。
  - `u32` のため負数は型レベルで受理されない（負数文字列は Axum デシリアライズエラー＝400）。
  - 正規化は純関数 `normalize_pagination(page: Option<u32>, limit: Option<u32>) -> (u32, u32)` としてハンドラ層に実装し、ユニットテスト対象とする。
  - *根拠*: TASK-0010 完了条件「limit デフォルト20・最大100」、要件定義 3章 境界値方針（🟡）、タスク仕様「limit は 1〜100 にクランプ」。

### 確定3: tag_id / category_id 絞り込みは EXISTS サブクエリ 🟡
- `tag_id` 指定時: `EXISTS (SELECT 1 FROM item_tags it WHERE it.item_id = items.id AND it.tag_id = $n)`
- `category_id` 指定時: `EXISTS (SELECT 1 FROM item_categories ic WHERE ic.item_id = items.id AND ic.category_id = $n)`
- JOIN + DISTINCT ではなく EXISTS を採用（多対多の重複排除が不要で `sqlx::QueryBuilder` と相性が良い）。
  - *根拠*: 要件定義 3章 DB制約（`EXISTS`/`IN` を許容）、note.md 6章 検討項目、本タスク文脈指示。

---

## 1. テスト戦略とレイヤー分担

| レイヤー | 対象 | テスト種別 | 実DB要否 |
|---|---|---|---|
| models/response.rs | `PaginatedOk<T>` シリアライズ・ステータス | ユニット | 不要 |
| handlers/items.rs | `normalize_pagination`（page/limitクランプ・OFFSET算出） | ユニット | 不要 |
| repositories/item_repository.rs | 動的WHERE句のSQL文字列生成（QueryBuilder） | ユニット | 不要 |
| repositories/item_repository.rs | `list_items` / `count_items` の実データ取得 | 統合 | 必要 |

- **ユニットテスト**: `#[test]` / `#[tokio::test]` をソースファイル内 `#[cfg(test)] mod tests` に同居（既存規約）。
- **統合テスト**: docker-compose のテスト用DB（マイグレーション適用済み）に対して `#[tokio::test]` で実行。`#[ignore]` 付与で通常 `cargo test` から分離し、`cargo test -- --ignored` で実行する方針（環境非依存化）。

---

## 2. 正常系テストケース（基本的な動作）

### TC-0010-N01: 絞り込みなしの一覧取得（デフォルトページネーション）
- **何をテストするか**: クエリパラメータなしの `GET /items` で、page=1/limit=20 が適用され先頭20件 + 総件数が返ること。
  - **期待される動作**: 全件のうち先頭20件を `data` に、総件数を `pagination.total` に格納して 200 を返す。
- **入力値**: クエリパラメータなし（事前データ: items を25件投入）。
  - **入力データの意味**: limit(20) を超える件数を用意し「先頭20件のみ返る」「total が全件数」を同時に検証する代表ケース。
- **期待される結果**: HTTPステータス200、`data.len() == 20`、`pagination == { page: 1, limit: 20, total: 25 }`、`success == true`。
  - **期待結果の理由**: 要件 2.2／TASK-0010 完了条件「デフォルト page=1, limit=20」「先頭20件」「total は同条件 COUNT(*)」に対応。
- **テストの目的**: 既定動作とレスポンスフォーマットの基本契約を確認する。
  - **確認ポイント**: `data` 件数が limit と一致、`total` が limit に依存せず全件数であること。
- **レイヤー**: 統合（実DB）。
- 🔵 信頼性レベル: TASK-0010 単体テスト要件TC-001・要件 UC-1 に直接対応。

### TC-0010-N02: media_type による絞り込み
- **何をテストするか**: `media_type=anime` 指定時、anime のitemのみが返ること。
  - **期待される動作**: WHERE 句に `media_type = $1` が追加され、anime 以外が除外される。
- **入力値**: `?media_type=anime`（事前データ: anime 3件 + movie 2件）。
  - **入力データの意味**: 複数種別が混在する状況で対象種別のみ抽出できるかを検証。
- **期待される結果**: 200、`data.len() == 3`、全要素の `media_type == "anime"`、`pagination.total == 3`。
  - **期待結果の理由**: 要件 UC-2／完了条件「絞り込み」「total は絞り込み後件数」に対応。
- **テストの目的**: 単一フィルタの適用と `total` が絞り込み後件数になることを確認する。
  - **確認ポイント**: `total` が全件数(5)ではなく絞り込み後(3)であること（COUNT が同条件で走る）。
- **レイヤー**: 統合（実DB）。
- 🔵 信頼性レベル: TASK-0010 単体テスト要件TC-002・要件 UC-2 に直接対応。

### TC-0010-N03: 複数条件の AND 絞り込み
- **何をテストするか**: `media_type=anime&is_favorite=true` で両条件を満たすitemのみが返ること。
  - **期待される動作**: WHERE 句に `media_type = $1 AND is_favorite = $2` が追加される。
- **入力値**: `?media_type=anime&is_favorite=true`（事前データ: anime/fav=true 2件, anime/fav=false 2件, movie/fav=true 1件）。
  - **入力データの意味**: 「片方だけ満たす」「両方満たす」「どちらも別」を混在させ AND 結合の正しさを検証。
- **期待される結果**: 200、`data.len() == 2`、全要素が `media_type=="anime"` かつ `is_favorite==true`、`pagination.total == 2`。
  - **期待結果の理由**: 要件 UC-3／完了条件「各フィルタは AND 結合」に対応。
- **テストの目的**: 複数フィルタが OR ではなく AND で結合されることを確認する。
  - **確認ポイント**: anime/fav=false（2件）と movie/fav=true（1件）が除外されること。
- **レイヤー**: 統合（実DB）。
- 🟡 信頼性レベル: TASK-0010 単体テスト要件TC-003・要件 UC-3 に対応（具体データ件数は妥当な推測）。

### TC-0010-N04: status による絞り込み
- **何をテストするか**: `status=in_progress` 指定時、該当ステータスのitemのみが返ること（`idx_items_status` 活用クエリ）。
  - **期待される動作**: WHERE 句に `status = $1` が追加される。
- **入力値**: `?status=in_progress`（事前データ: in_progress 2件, not_started 1件, completed 1件）。
  - **入力データの意味**: 3種ステータス混在で対象のみ抽出できるかを検証。
- **期待される結果**: 200、`data.len() == 2`、全要素の `status == "in_progress"`、`pagination.total == 2`。
  - **期待結果の理由**: 完了条件「status フィルタ」「idx_items_status 活用」、入力仕様表に対応。
- **テストの目的**: status フィルタ単体の動作を確認する。
  - **確認ポイント**: not_started / completed が除外されること。
- **レイヤー**: 統合（実DB）。
- 🔵 信頼性レベル: 要件 入力仕様表（status）・完了条件に直接対応。

### TC-0010-N05: is_favorite による絞り込み
- **何をテストするか**: `is_favorite=true` 指定時、お気に入りitemのみが返ること（`idx_items_is_favorite` 活用）。
- **入力値**: `?is_favorite=true`（事前データ: fav=true 3件, fav=false 2件）。
  - **入力データの意味**: bool フィルタの true 絞り込みを単体検証。
- **期待される結果**: 200、`data.len() == 3`、全要素 `is_favorite == true`、`pagination.total == 3`。
  - **期待結果の理由**: 要件 入力仕様表（is_favorite）・完了条件に対応。
- **テストの目的**: bool フィルタ単体の動作を確認する。
  - **確認ポイント**: fav=false が除外されること。
- **レイヤー**: 統合（実DB）。
- 🔵 信頼性レベル: 要件 入力仕様表（is_favorite）に直接対応。

### TC-0010-N06: tag_id による絞り込み（EXISTS サブクエリ）
- **何をテストするか**: `tag_id=<uuid>` 指定時、`item_tags` 経由で当該タグを持つitemのみが返ること。
  - **期待される動作**: WHERE 句に `EXISTS (SELECT 1 FROM item_tags it WHERE it.item_id = items.id AND it.tag_id = $n)` が追加される。
- **入力値**: `?tag_id=<TAG_A>`（事前データ: TAG_A 紐付け 2件、TAG_B 紐付け 1件、タグなし 1件）。
  - **入力データの意味**: 多対多関係でタグ一致itemのみ抽出し、重複行が出ない（EXISTS）ことを検証。
- **期待される結果**: 200、`data.len() == 2`（重複なし）、`pagination.total == 2`。
  - **期待結果の理由**: 確定3（EXISTS）・要件 UC-4・統合テスト要件に対応。
- **テストの目的**: 中間テーブル絞り込みと重複排除を確認する。
  - **確認ポイント**: 同一itemが複数タグを持っても data に1回のみ出現すること（EXISTS なので重複しない）。
- **レイヤー**: 統合（実DB）。
- 🟡 信頼性レベル: 要件 UC-4・統合テスト要件に対応（SQL形状は確定3の推測）。

### TC-0010-N07: category_id による絞り込み（EXISTS サブクエリ）
- **何をテストするか**: `category_id=<uuid>` 指定時、`item_categories` 経由で当該カテゴリのitemのみが返ること。
  - **期待される動作**: WHERE 句に `EXISTS (SELECT 1 FROM item_categories ic WHERE ic.item_id = items.id AND ic.category_id = $n)` が追加される。
- **入力値**: `?category_id=<CAT_A>`（事前データ: CAT_A 紐付け 2件、CAT_B 1件、カテゴリなし 1件）。
- **期待される結果**: 200、`data.len() == 2`、`pagination.total == 2`。
  - **期待結果の理由**: 確定3・要件 UC-5・統合テスト要件に対応。
- **テストの目的**: カテゴリ中間テーブル絞り込みを確認する。
  - **確認ポイント**: 重複なし、対象カテゴリ外が除外されること。
- **レイヤー**: 統合（実DB）。
- 🟡 信頼性レベル: 要件 UC-5・統合テスト要件に対応（SQL形状は確定3の推測）。

### TC-0010-N08: tag_id と media_type の AND 複合（中間テーブル + 通常カラム）
- **何をテストするか**: `media_type=anime&tag_id=<TAG_A>` で、通常カラムフィルタと EXISTS サブクエリが AND 結合されること。
- **入力値**: `?media_type=anime&tag_id=<TAG_A>`（事前データ: anime+TAG_A 1件、anime+TAG_B 1件、movie+TAG_A 1件）。
  - **入力データの意味**: 種別とタグの組合せで、両方満たす1件のみが返るかを検証。
- **期待される結果**: 200、`data.len() == 1`、`pagination.total == 1`。
  - **期待結果の理由**: 完了条件「各フィルタ AND 結合」＋確定3 を組み合わせた検証。
- **テストの目的**: 異種フィルタ（カラム条件 + サブクエリ条件）の AND 共存を確認する。
  - **確認ポイント**: anime+TAG_B と movie+TAG_A の双方が除外されること。
- **レイヤー**: 統合（実DB）。
- 🟡 信頼性レベル: 完了条件 + 確定3 からの妥当な推測。

### TC-0010-N09: PaginatedOk<T> のJSONシリアライズ形式
- **何をテストするか**: 新規 `PaginatedOk<T>` が `{ "success": true, "data": [...], "pagination": { "page", "limit", "total" } }` 形式でシリアライズされること。
  - **期待される動作**: serde 出力が要件 2.2 のフォーマットに一致。
- **入力値**: `PaginatedOk::new(vec![sample_item()], Pagination { page: 1, limit: 20, total: 100 })`。
  - **入力データの意味**: 既存 `ApiOk` のシリアライズテスト（`api_ok_serializes_to_expected_json`）に倣った最小データ。
- **期待される結果**: `serde_json::to_value` の結果が `{"success":true,"data":[...],"pagination":{"page":1,"limit":20,"total":100}}` と一致。
  - **期待結果の理由**: 確定1・要件 2.2／完了条件「レスポンス形式」に対応。
- **テストの目的**: レスポンス契約（キー名・構造）を実DBなしで固定する。
  - **確認ポイント**: トップレベルキーが `success`/`data`/`pagination` の3つであること、`pagination` のキー名が `page`/`limit`/`total` であること。
- **レイヤー**: ユニット（実DB不要、`models/response.rs`）。
- 🟡 信頼性レベル: 要件 2.2 と `ApiOk` 既存テストからの妥当な推測（型は確定1で新規定義）。

### TC-0010-N10: PaginatedOk<T> が HTTP 200 を返す
- **何をテストするか**: `PaginatedOk<T>::into_response()` がステータス200を返すこと。
- **入力値**: `PaginatedOk::new(Vec::<Item>::new(), Pagination { page: 1, limit: 20, total: 0 })`。
- **期待される結果**: `response.status() == StatusCode::OK`。
  - **期待結果の理由**: 要件 2.2「HTTPステータス 200 OK」、`ApiOk` の 200固定規約に対応。
- **テストの目的**: 成功時ステータスコードを固定する。
  - **確認ポイント**: 空配列でも 200（404 等にしない）。
- **レイヤー**: ユニット（`models/response.rs`）。
- 🔵 信頼性レベル: 要件 2.2・既存 `ApiOk` IntoResponse 規約に直接対応。

---

## 3. 異常系テストケース（エラーハンドリング）

### TC-0010-E01: 不正な media_type 値 → 400
- **エラーケースの概要**: enum に存在しない `media_type=invalid` を指定。
  - **エラー処理の重要性**: 不正な列挙値で SQL を組み立てず、入力段階で弾く必要がある。
- **入力値**: `?media_type=invalid`。
  - **不正な理由**: `MediaType` enum（anime/movie/.../paper）に存在しない値。
  - **実際の発生シナリオ**: クライアントのタイプミス・古いフロントからの不正値送信。
- **期待される結果**: Axum の `Query` デシリアライズエラーで **400 Bad Request**。
  - **エラーメッセージの内容**: リクエストパラメータが不正である旨（Axum 既定 or VALIDATION_ERROR）。
  - **システムの安全性**: DBクエリに到達せずに拒否される。
- **テストの目的**: 不正列挙値の入力検証を確認する。
  - **品質保証の観点**: 型安全による入力ガードが機能していること。
- **レイヤー**: 統合（ルーター経由）またはハンドラ単体（Query抽出失敗の確認）。
- 🔵 信頼性レベル: 要件 EC-1・TASK-0010 注意事項に直接対応。

### TC-0010-E02: 不正な page 値（非数値）→ 400
- **エラーケースの概要**: `page=abc` のような `u32` にパースできない値を指定。
- **入力値**: `?page=abc`。
  - **不正な理由**: `u32` 型へデシリアライズできない文字列。
  - **実際の発生シナリオ**: 手書きURL・不正なクエリ生成。
- **期待される結果**: **400 Bad Request**（Axum デシリアライズエラー）。
  - **システムの安全性**: 不正値で OFFSET 計算に進まない。
- **テストの目的**: 数値パラメータの型検証を確認する。
  - **品質保証の観点**: クランプ対象（page=0 等）と「型不正（abc）」を区別して扱えていること。
- **レイヤー**: 統合（ルーター経由）。
- 🔵 信頼性レベル: 要件 EC-1・note.md 6章 技術的制約に直接対応。

### TC-0010-E03: 不正な is_favorite 値（bool以外）→ 400
- **エラーケースの概要**: `is_favorite=yes` のような bool に解釈できない値を指定。
- **入力値**: `?is_favorite=yes`。
  - **不正な理由**: `bool` へデシリアライズできない（`true`/`false` 以外）。
- **期待される結果**: **400 Bad Request**。
  - **システムの安全性**: 曖昧な真偽値で WHERE 条件を組まない。
- **テストの目的**: bool パラメータの型検証を確認する。
  - **品質保証の観点**: 入力仕様（true/false のみ）を厳守していること。
- **レイヤー**: 統合（ルーター経由）。
- 🟡 信頼性レベル: 要件 入力仕様表（is_favorite: true/false）からの妥当な推測。

### TC-0010-E04: DBエラー時 → 500 INTERNAL_ERROR
- **エラーケースの概要**: DB接続障害・クエリ失敗時に内部エラーへ変換されること。
  - **エラー処理の重要性**: DB内部情報を外部へ漏らさず汎用エラーを返す（既存 `db_error` 方針）。
- **入力値**: 正常なクエリだが、DBが利用不可（プール切断 / 不正な接続文字列で意図的に失敗を誘発）。
  - **実際の発生シナリオ**: DBダウン・ネットワーク断・マイグレーション未適用。
- **期待される結果**: **500**、`error.code == "INTERNAL_ERROR"`、メッセージは汎用文言（スキーマ詳細を含まない）。
  - **エラーメッセージの内容**: ユーザー向けは固定の汎用文言、詳細は `tracing::error!` でログのみ。
  - **システムの安全性**: 内部スキーマ・SQL詳細がレスポンスに漏れないこと。
- **テストの目的**: DBエラーの統一エラー変換を確認する。
  - **品質保証の観点**: 情報漏洩防止（既存 `db_error` セキュリティ方針）の継続。
- **レイヤー**: 統合（DB障害を再現可能な場合）。再現困難時は `list_items` の `Err` 経路を `db_error` 経由で確認する単体に縮退。
- 🟡 信頼性レベル: 要件 EC-2・既存 `db_error` 関数からの妥当な推測。

---

## 4. 境界値テストケース（クランプ・空・最大）

### TC-0010-B01: limit 最大値クランプ（limit=500 → 100）
- **境界値の意味**: 上限 100 を超える指定が 100 に丸められる境界。
  - **境界値での動作保証**: 上限超過でもエラーにせず安全に上限値で動作。
- **入力値**: `normalize_pagination(Some(1), Some(500))`。
  - **境界値選択の根拠**: TASK-0010 TC-004 が `limit=500` を明示。
  - **実際の使用場面**: クライアントが大量取得を試みた場合のサーバー側保護。
- **期待される結果**: `(page, limit) == (1, 100)`。レスポンスの `pagination.limit == 100`、`data.len() <= 100`。
  - **境界での正確性**: LIMIT 句に 100 がバインドされる。
  - **一貫した動作**: 100超は常に100、100以下はそのまま。
- **テストの目的**: 上限クランプを確認する（純関数で実DB不要）。
  - **堅牢性の確認**: 過大要求でもメモリ・応答時間が保護されること。
- **レイヤー**: ユニット（`normalize_pagination`）。
- 🟡 信頼性レベル: TASK-0010 TC-004・要件 UC-6 に対応（純関数化は確定2）。

### TC-0010-B02: limit 上限ちょうど（limit=100 → 100）
- **境界値の意味**: クランプ境界の内側ぴったり（100）。
- **入力値**: `normalize_pagination(Some(1), Some(100))`。
  - **境界値選択の根拠**: 上限 100 が「クランプされず通過する」境界の確認。
- **期待される結果**: `(1, 100)`（クランプ非発生）。
  - **一貫した動作**: 101 はクランプ、100 は非クランプ（off-by-one 防止）。
- **テストの目的**: 上限境界の包含関係（`> 100` でクランプ、`== 100` は通過）を確認する。
- **レイヤー**: ユニット。
- 🟡 信頼性レベル: 確定2・タスク「1〜100にクランプ」からの妥当な推測。

### TC-0010-B03: limit=0 → デフォルト20にクランプ
- **境界値の意味**: 下限割れ（0）はエラーではなくデフォルトに丸める境界。
- **入力値**: `normalize_pagination(Some(1), Some(0))`。
  - **境界値選択の根拠**: 要件 未確定事項2・確定2「limit<1 → 20」。
- **期待される結果**: `(1, 20)`。
  - **境界での正確性**: LIMIT 0 で空結果を返すのではなくデフォルト適用。
- **テストの目的**: 下限クランプ（0 → 20）を確認する。
  - **堅牢性の確認**: limit=0 の無意味クエリを防ぐ。
- **レイヤー**: ユニット。
- 🟡 信頼性レベル: 確定2（本フェーズで方針確定）に基づく。

### TC-0010-B04: page=0 → 1にクランプ（OFFSET=0）
- **境界値の意味**: 1未満の page は 1 に丸める境界。
- **入力値**: `normalize_pagination(Some(0), Some(20))`。
  - **境界値選択の根拠**: 確定2「page<1 → 1」。`OFFSET=(page-1)*limit` で page=0 だと負/オーバーフローになり得るため要クランプ。
- **期待される結果**: `(1, 20)`。算出 `OFFSET == 0`。
  - **境界での正確性**: `u32` の `(0-1)` アンダーフロー回避を保証。
- **テストの目的**: 下限 page クランプと OFFSET 安全算出を確認する。
  - **堅牢性の確認**: アンダーフロー panic を起こさないこと。
- **レイヤー**: ユニット。
- 🟡 信頼性レベル: 確定2・note.md 6章（page=0 方針）に基づく。

### TC-0010-B05: パラメータ未指定 → デフォルト(page=1, limit=20)
- **境界値の意味**: `None` 入力時のデフォルト適用境界。
- **入力値**: `normalize_pagination(None, None)`。
- **期待される結果**: `(1, 20)`、`OFFSET == 0`。
  - **一貫した動作**: 未指定とデフォルト値指定で同一結果。
- **テストの目的**: Option の None → デフォルトフォールバックを確認する。
- **レイヤー**: ユニット。
- 🔵 信頼性レベル: 要件 入力仕様表（page デフォルト1, limit デフォルト20）に直接対応。

### TC-0010-B06: OFFSET 算出（page=2, limit=20 → OFFSET=20）
- **境界値の意味**: 2ページ目の先頭オフセット境界。
- **入力値**: page=2, limit=20。
  - **境界値選択の根拠**: 要件 UC-7「page=2&limit=20 → 21〜40件目（OFFSET=20）」。
- **期待される結果**: `OFFSET == (2-1)*20 == 20`。統合では 21〜40件目に相当する `data`。
  - **境界での正確性**: `(page-1)*limit` の算出が正しいこと。
- **テストの目的**: ページ送りの OFFSET 計算を確認する。
- **レイヤー**: ユニット（算出）＋統合（実データ確認）。
- 🟡 信頼性レベル: 要件 UC-7 に対応。

### TC-0010-B07: 範囲外 page → 空配列 + 正しい total
- **境界値の意味**: 件数を超える page 指定の境界（データ枯渇）。
- **入力値**: 事前データ 5件に対し `?page=10&limit=20`（OFFSET=180）。
  - **境界値選択の根拠**: 要件 UC-8「件数を超える page は空配列 + 正しい total」。
- **期待される結果**: 200、`data == []`、`pagination == { page: 10, limit: 20, total: 5 }`。
  - **一貫した動作**: データがなくても total は全条件件数を返す（404 にしない）。
- **テストの目的**: オーバーフローpage時の挙動を確認する。
  - **堅牢性の確認**: 空でもエラーにせず正常レスポンスを返すこと。
- **レイヤー**: 統合（実DB）。
- 🟡 信頼性レベル: 要件 UC-8 に対応。

### TC-0010-B08: 全件0件（空テーブル）→ data=[], total=0
- **境界値の意味**: データ最小（0件）の境界。
- **入力値**: items 0件で `GET /items`。
- **期待される結果**: 200、`data == []`、`pagination == { page: 1, limit: 20, total: 0 }`。
  - **境界での正確性**: COUNT(*)=0 が `total` に正しく反映される。
- **テストの目的**: データ皆無時のフォーマット維持を確認する。
- **レイヤー**: 統合（実DB）。
- 🟡 信頼性レベル: 要件 2.2 から妥当な推測（空集合の自明ケース）。

---

## 5. リポジトリ層 SQL生成のユニットテスト（QueryBuilder検証）

> 実DB非依存で、動的WHERE句の構造（EXISTS・AND結合・LIMIT/OFFSET）を SQL文字列として検証する。`QueryBuilder::sql()` で生成SQLを取得し部分一致で確認する設計を想定。

### TC-0010-Q01: フィルタなし時のSQLにWHERE句が付かない
- **入力値**: 全フィルタ `None`。
- **期待される結果**: 生成SQLが `SELECT ... FROM items` ベースで `WHERE` を含まない（または常に真のプレースホルダなし）、末尾に `LIMIT`/`OFFSET` を含む。
- **テストの目的**: 不要なWHERE句が付かないことを確認する。
- 🟡 信頼性レベル: 確定3・QueryBuilder方針からの妥当な推測。

### TC-0010-Q02: media_type 指定時のSQLに `media_type = ` を含む
- **入力値**: `media_type=Some(Anime)` のみ。
- **期待される結果**: 生成SQLに `media_type = ` を含む（バインド値はパラメータ化）。
- **テストの目的**: 単一カラムフィルタの句生成を確認する。
- 🔵 信頼性レベル: 完了条件「media_type 絞り込み」に対応。

### TC-0010-Q03: tag_id 指定時のSQLに item_tags の EXISTS を含む
- **入力値**: `tag_id=Some(uuid)` のみ。
- **期待される結果**: 生成SQLに `EXISTS` および `item_tags` を含む。
- **テストの目的**: 確定3のEXISTSパターン生成を確認する。
- 🟡 信頼性レベル: 確定3に基づく。

### TC-0010-Q04: category_id 指定時のSQLに item_categories の EXISTS を含む
- **入力値**: `category_id=Some(uuid)` のみ。
- **期待される結果**: 生成SQLに `EXISTS` および `item_categories` を含む。
- **テストの目的**: 確定3のEXISTSパターン生成を確認する。
- 🟡 信頼性レベル: 確定3に基づく。

### TC-0010-Q05: 複数フィルタ時にSQLが AND で結合される
- **入力値**: `media_type=Some(Anime)`, `is_favorite=Some(true)`。
- **期待される結果**: 生成SQLに `AND` を含み、両カラム条件が連結される。
- **テストの目的**: AND結合の句生成を確認する。
- 🟡 信頼性レベル: 完了条件「AND結合」＋QueryBuilder方針からの妥当な推測。

### TC-0010-Q06: list_items と count_items が同一WHERE句を共有
- **入力値**: 同一フィルタ集合を両関数に渡す。
- **期待される結果**: 生成された WHERE 部分（フィルタ句）が両者で一致する（`total` が `data` と同条件であることの保証）。
- **テストの目的**: total の整合性（同条件COUNT）をSQLレベルで保証する。
- 🟡 信頼性レベル: 要件 2.2/3章「total は同条件 COUNT(*)」からの妥当な推測。

---

## 6. 開発言語・テストフレームワーク

- **プログラミング言語**: Rust
  - **言語選択の理由**: 既存バックエンド（mediavault-api）が Rust + Axum + sqlx で実装済みであり、本タスクもその継続。
  - **テストに適した機能**: 型安全な enum（MediaType/ItemStatus）で入力検証が静的に保証され、`#[cfg(test)]` モジュールで同居テストが容易。
- **テストフレームワーク**: Rust 標準テスト（`#[test]` / `#[tokio::test]`）
  - **フレームワーク選択の理由**: 既存テスト（response.rs, item.rs, items.rs, item_repository.rs）が標準テストで統一されており、追加依存なしで一貫性を保てる。
  - **テスト実行環境**:
    - ユニット: `cargo test -p mediavault-api`（実DB不要）。
    - 統合: docker-compose のテスト用Postgres（マイグレーション適用済み）に対し `#[ignore]` 付きテストを `cargo test -- --ignored` で実行。`cd backend && docker compose up -d db` 前提。
- 🔵 信頼性レベル: note.md 5章 テスト関連情報・`backend/CLAUDE.md` のコマンドに直接対応。

---

## 7. テストケース実装時の日本語コメント指針（例）

### ユニット例: normalize_pagination のクランプ（TC-0010-B01）

```rust
#[test]
fn normalize_pagination_clamps_limit_to_100() {
    // 【テスト目的】: limit が上限(100)を超えた場合に 100 へクランプされることを確認する
    // 【テスト内容】: normalize_pagination(Some(1), Some(500)) を呼び出す
    // 【期待される動作】: (page, limit) = (1, 100) が返る
    // 🟡 信頼性レベル: TASK-0010 TC-004（limit=500→100）に対応

    // 【テストデータ準備】: 上限超過の limit=500 を用意（過大要求のサーバー保護を検証するため）
    // 【初期条件設定】: page は正常値 1
    let (page, limit) = normalize_pagination(Some(1), Some(500));

    // 【結果検証】: limit が 100 に丸められること
    // 【品質保証】: 過大要求でも応答時間・メモリが保護される
    assert_eq!(page, 1);   // 【検証項目】: page は変更されない 🟡
    assert_eq!(limit, 100); // 【検証項目】: limit が上限100にクランプされる 🟡
}
```

### ユニット例: PaginatedOk のシリアライズ（TC-0010-N09）

```rust
#[test]
fn paginated_ok_serializes_to_expected_json() {
    // 【テスト目的】: PaginatedOk が {success, data, pagination} 形式でシリアライズされることを確認する
    // 🟡 信頼性レベル: 確定1・要件2.2 のレスポンス形式に対応
    let body = PaginatedOk::new(
        vec![serde_json::json!({"id": 1})],
        Pagination { page: 1, limit: 20, total: 100 },
    );
    let json = serde_json::to_value(&body).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "success": true,
            "data": [{"id": 1}],
            "pagination": {"page": 1, "limit": 20, "total": 100}
        })
    ); // 【検証項目】: トップレベルキーと pagination 構造が要件通りであること 🟡
}
```

### 統合例: 絞り込みなし一覧（TC-0010-N01, 抜粋）

```rust
#[tokio::test]
#[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
async fn list_items_returns_first_20_with_total() {
    // 【テスト前準備】: テスト用DBへ items を25件投入する（マイグレーション適用済み前提）
    // 【環境初期化】: トランザクション or 事前TRUNCATEでクリーンな状態にする
    // ... seed 25 items ...

    // 【実際の処理実行】: list_items + count_items を既定ページネーションで呼ぶ
    let (items, total) = /* list_items(&pool, &ListItemsQuery::default()).await */;

    // 【結果検証】: 先頭20件 + total=25
    assert_eq!(items.len(), 20); // 【検証項目】: limit=20 で先頭20件のみ取得 🔵
    assert_eq!(total, 25);       // 【検証項目】: total は limit に依存せず全件数 🔵
}
```

---

## 8. 要件定義との対応関係

- **参照した機能概要**: 要件定義 1章（GET /items の絞り込み・ページネーション）
- **参照した入力・出力仕様**: 要件定義 2.1（クエリパラメータ表）, 2.2（成功レスポンス）, 2.3（エラーレスポンス）
- **参照した制約条件**: 要件定義 3章（パフォーマンス/セキュリティ/アーキテクチャ/DB/API制約、境界値方針）
- **参照した使用例**: 要件定義 4章（UC-1〜UC-8, EC-1〜EC-2）
- **参照した単体テスト要件**: TASK-0010.md TC-001（絞り込みなし）, TC-002（media_type）, TC-003（AND）, TC-004（limitクランプ）
- **参照した統合テスト要件**: TASK-0010.md（tag_id/category_id の実DB統合テスト）
- **参照した既存実装**: `models/response.rs`（ApiOk）, `models/item.rs`（Item/MediaType/ItemStatus）, `handlers/items.rs`（created_response パターン）, `repositories/item_repository.rs`（db_error/detail_table_name）

---

## 9. テストケース一覧（トレーサビリティ）

| ID | 分類 | 概要 | レイヤー | 対応要件 | 信頼性 |
|---|---|---|---|---|---|
| TC-0010-N01 | 正常 | 絞り込みなし（デフォルトページング） | 統合 | TC-001/UC-1 | 🔵 |
| TC-0010-N02 | 正常 | media_type 絞り込み | 統合 | TC-002/UC-2 | 🔵 |
| TC-0010-N03 | 正常 | media_type + is_favorite の AND | 統合 | TC-003/UC-3 | 🟡 |
| TC-0010-N04 | 正常 | status 絞り込み | 統合 | 完了条件 | 🔵 |
| TC-0010-N05 | 正常 | is_favorite 絞り込み | 統合 | 入力仕様 | 🔵 |
| TC-0010-N06 | 正常 | tag_id 絞り込み（EXISTS） | 統合 | UC-4 | 🟡 |
| TC-0010-N07 | 正常 | category_id 絞り込み（EXISTS） | 統合 | UC-5 | 🟡 |
| TC-0010-N08 | 正常 | tag_id + media_type の AND 複合 | 統合 | 完了条件/確定3 | 🟡 |
| TC-0010-N09 | 正常 | PaginatedOk シリアライズ形式 | ユニット | 2.2/確定1 | 🟡 |
| TC-0010-N10 | 正常 | PaginatedOk が 200 を返す | ユニット | 2.2 | 🔵 |
| TC-0010-E01 | 異常 | 不正 media_type → 400 | 統合/HD | EC-1 | 🔵 |
| TC-0010-E02 | 異常 | 不正 page（非数値）→ 400 | 統合 | EC-1 | 🔵 |
| TC-0010-E03 | 異常 | 不正 is_favorite → 400 | 統合 | 入力仕様 | 🟡 |
| TC-0010-E04 | 異常 | DBエラー → 500 INTERNAL_ERROR | 統合/縮退 | EC-2 | 🟡 |
| TC-0010-B01 | 境界 | limit=500 → 100 クランプ | ユニット | TC-004/UC-6 | 🟡 |
| TC-0010-B02 | 境界 | limit=100 → 100（非クランプ境界） | ユニット | 確定2 | 🟡 |
| TC-0010-B03 | 境界 | limit=0 → 20 クランプ | ユニット | 確定2 | 🟡 |
| TC-0010-B04 | 境界 | page=0 → 1 クランプ（OFFSET=0） | ユニット | 確定2 | 🟡 |
| TC-0010-B05 | 境界 | 未指定 → (1,20) デフォルト | ユニット | 入力仕様 | 🔵 |
| TC-0010-B06 | 境界 | page=2,limit=20 → OFFSET=20 | ユニット/統合 | UC-7 | 🟡 |
| TC-0010-B07 | 境界 | 範囲外 page → 空配列 + total | 統合 | UC-8 | 🟡 |
| TC-0010-B08 | 境界 | 0件 → data=[], total=0 | 統合 | 2.2 | 🟡 |
| TC-0010-Q01 | SQL | フィルタなし → WHERE句なし | ユニット | 確定3 | 🟡 |
| TC-0010-Q02 | SQL | media_type → `media_type = ` | ユニット | 完了条件 | 🔵 |
| TC-0010-Q03 | SQL | tag_id → item_tags EXISTS | ユニット | 確定3 | 🟡 |
| TC-0010-Q04 | SQL | category_id → item_categories EXISTS | ユニット | 確定3 | 🟡 |
| TC-0010-Q05 | SQL | 複数フィルタ → AND 結合 | ユニット | 完了条件 | 🟡 |
| TC-0010-Q06 | SQL | list/count が同一WHERE共有 | ユニット | 2.2 total整合 | 🟡 |

**合計**: 28ケース（正常10 / 異常4 / 境界8 / SQL生成6）

---

## 10. 品質判定

```
✅ 高品質:
- テストケース分類: 正常系(10)・異常系(4)・境界値(8)・SQL生成(6)を網羅
- 期待値定義: 各ケースに具体的な期待値（件数・ステータス・JSON構造・SQL断片）を明記
- 技術選択: Rust + 標準テスト（#[test]/#[tokio::test]）で確定、実DBは docker-compose で分離
- 実装可能性: 既存パターン（ApiOk/db_error/QueryBuilder）に準拠し実現可能
- 未確定3項目を本フェーズで確定（PaginatedOk型 / クランプ方針 / EXISTS）
- 信頼性レベル分布: 🔵 中心（主要要件は設計文書・タスク・既存実装に裏付け）、🟡 は本フェーズ確定事項由来、🔴 なし
```

**信頼性レベル分布**:
- 🔵 青信号: 9ケース（基本ユースケース・入力仕様・エラー400・既存規約由来）
- 🟡 黄信号: 19ケース（本フェーズ確定事項=PaginatedOk/クランプ/EXISTS、AND複合、具体データ件数）
- 🔴 赤信号: 0ケース

---

## 次のお勧めステップ

`/tsumiki:tdd-red mediavault-backend TASK-0010` で Redフェーズ（失敗テスト作成）を開始します。
- まず実DB不要のユニットテスト（TC-0010-N09/N10, B01〜B06, Q01〜Q06）から着手すると、`PaginatedOk` 型・`normalize_pagination`・QueryBuilder の設計を早期に固定できる。
- 統合テスト（N01〜N08, B07/B08, E01〜E04）は docker-compose テスト用DB前提で `#[ignore]` 運用とする。
