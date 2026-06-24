# TASK-0015 Greenフェーズ記録: タグ・カテゴリCRUD実装

## 実装方針

Redフェーズで作成したテストを通すため、以下のファイルへ実装本体を追加した。`item.rs`/`item_repository.rs`/`items.rs`の既存パターン（DTO・バリデーション・`db_error`・SQLSTATE別エラーハンドリング・ハンドラ・ルーティング）を忠実に踏襲し、タグ・カテゴリで完全に対称な構造とした。

## 実装したファイル

### backend/mediavault-api/src/models/response.rs（追記）
- `ApiErrorCode`へ`DuplicateTagName`(409)・`TagNotFound`(404)・`DuplicateCategoryName`(409)・`CategoryNotFound`(404)を追加

### backend/mediavault-api/src/models/tag.rs
- `Tag { id, name }`（`sqlx::FromRow`+`Serialize`+`Deserialize`）
- `CreateTagRequest { name }`
- `validate_tag_name(name: &str) -> Result<(), ApiError>`（trim().is_empty()チェックのみ。最大長はDB制約に委ねる）

### backend/mediavault-api/src/models/category.rs
- tag.rsと完全に対称（`Category`, `CreateCategoryRequest`, `validate_category_name`）

### backend/mediavault-api/src/repositories/tag_repository.rs
- `db_error`, `is_unique_violation`(23505), `is_foreign_key_violation`(23503)
- `create_tag`: INSERT、一意制約違反→`DuplicateTagName`(409)
- `delete_tag`: DELETE、影響行数で存在確認（呼び出し側で404判定）
- `attach_tag_to_item`: item_tagsへ複合キーINSERT、FK違反→`TagNotFound`(404)、PK違反→`DuplicateTagName`(409)
- `detach_tag_from_item`: item_tagsから複合キーDELETE

### backend/mediavault-api/src/repositories/category_repository.rs
- tag_repository.rsと完全に対称

### backend/mediavault-api/src/handlers/tags.rs
- `create_tag_handler`（201）, `delete_tag_handler`（204/404）, `attach_tag_handler`（201）, `detach_tag_handler`（204/404）

### backend/mediavault-api/src/handlers/categories.rs
- tags.rsと完全に対称

### backend/mediavault-api/src/routes/mod.rs（追記）
```
.route("/tags", post(create_tag_handler))
.route("/tags/:id", delete(delete_tag_handler))
.route("/items/:id/tags/:tag_id", post(attach_tag_handler).delete(detach_tag_handler))
.route("/categories", post(create_category_handler))
.route("/categories/:id", delete(delete_category_handler))
.route("/items/:id/categories/:category_id", post(attach_category_handler).delete(detach_category_handler))
```

### モジュール登録
- `models/mod.rs`, `repositories/mod.rs`, `handlers/mod.rs` へ新規モジュールを追加

## テスト実行結果

```bash
cd backend
cargo test -p mediavault-api
```

```
test result: ok. 60 passed; 0 failed; 56 ignored; 0 measured; 0 filtered out
```

- Red フェーズで作成した全20テスト（model層7件は即時実行・成功、repository層13件はDB依存のため`#[ignore]`で保留、ルーター/ハンドラ層は本フェーズでは未追加）がコンパイル・実行成功
- 既存の全テスト（item系等）も regress なく成功

## ファイルサイズ

全新規ファイルは150行未満（800行制限に対し十分余裕あり）

## モック使用確認

実装コード（tag.rs, category.rs, tag_repository.rs, category_repository.rs, handlers/tags.rs, handlers/categories.rs）にモック・スタブ・インメモリストレージは一切使用していない。すべて実際のsqlx経由のPostgreSQL操作。

## 課題・改善点（Refactorフェーズで対応）

1. `tag_repository.rs`と`category_repository.rs`、`handlers/tags.rs`と`handlers/categories.rs`が完全に対称的なコード重複になっている。ジェネリクスやマクロでの共通化を検討する余地があるが、既存`item_repository.rs`の設計判断（明示的な重複を許容しシンプルさを優先）に合わせ、本タスクでは重複を許容する方針が妥当か検討する
2. `attach_tag_to_item`/`attach_category_to_item`の重複付与時エラーは現在`DuplicateTagName`/`DuplicateCategoryName`を流用しているが、本来は「付与の重複」用の専用コード（例: `TAG_ALREADY_ATTACHED`）に分けるべきか検討
3. HTTPレベルの統合テスト（ルーター経由）がまだ未追加。`routes/mod.rs`の既存テスト（`get_items_with_invalid_media_type_returns_400`等）と同様のパターンで追加を検討
4. 実DB統合テスト（13件、`#[ignore]`）はdocker-compose環境での実行確認がまだ済んでいない
