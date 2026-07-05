# item_files

`backend/mediavault-api/migrations/20260623000002_add_relation_tables.up.sql` / `backend/mediavault-api/src/models/item_file.rs`

## DBスキーマ

### item_files

| カラム | 型 | NULL | 備考 |
|---|---|---|---|
| id | UUID PK | NOT NULL | `gen_random_uuid()` |
| item_id | UUID FK → items(id) ON DELETE CASCADE | NOT NULL | |
| path | VARCHAR(1000) | NOT NULL | |
| label | VARCHAR(255) | NULL | |
| file_type | file_type | NOT NULL | pdf / image / other |
| calibre_book_id | VARCHAR(100) | NULL | Calibre-Web連携ID |
| created_at / updated_at | TIMESTAMP | NOT NULL | `updated_at`はトリガー`trg_item_files_updated_at`で自動更新 |

インデックス: `idx_item_files_item_id`

## Rustモデル

```rust
#[sqlx(type_name = "file_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum FileType { Pdf, Image, Other }
```

- `ItemFile { id, item_id, path, label: Option<String>, file_type, calibre_book_id: Option<String>, created_at }`（`sqlx::FromRow`）
- `CreateItemFileRequest { path, label, file_type }`（パス指定方式） — `parse_create_item_file_request`で`path`空文字を拒否。
- `UpdateCalibreLinkRequest { calibre_book_id: String }` — `PATCH /items/:id/files/:file_id/calibre-link`用。`parse_update_calibre_link_request`でtrim後空文字を拒否。
- `parse_file_id(raw: &str) -> Result<Uuid, ApiError>` — `:file_id`パスパラメータのUUIDパース（既存`item::parse_item_id`と対称）。
- `CalibreWebLinkInfo { file_id: Uuid, calibre_book_id: String }` — DB非対応の補助struct。`calibre_book_id`設定済みPDFの`item_files`について、`ItemDetail.calibre_links`に付加するCalibre-Web遷移情報を保持する（Calibre-Web側のURL構成・認証方式が未確定のため独立構造体として定義）。

## 参照

エンドポイント例は [mediavault-api/item-files.md](../mediavault-api/item-files.md) を参照。
