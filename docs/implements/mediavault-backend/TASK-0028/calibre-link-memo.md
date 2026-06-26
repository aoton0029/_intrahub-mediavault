# TDD開発メモ: calibre-link

## 概要

- 機能名: calibre-link（item_files の calibre_book_id 更新 + アイテム詳細APIへのCalibre-Web遷移情報付加）
- 開発開始: 2026-06-26
- 現在のフェーズ: Red（完了）

## 関連ファイル

- 元タスクファイル: `docs/tasks/mediavault-backend/TASK-0028.md`
- 要件定義: `docs/implements/mediavault-backend/TASK-0028/calibre-link-requirements.md`
- テストケース定義: `docs/implements/mediavault-backend/TASK-0028/calibre-link-testcases.md`
- 実装ファイル:
  - `backend/mediavault-api/src/models/response.rs`（ApiErrorCode::FileNotFound追加）
  - `backend/mediavault-api/src/models/item_file.rs`（UpdateCalibreLinkRequest, parse_update_calibre_link_request, parse_file_id, CalibreWebLinkInfo追加）
  - `backend/mediavault-api/src/repositories/item_file_repository.rs`（update_calibre_link, get_item_calibre_links追加）
  - `backend/mediavault-api/src/handlers/item_files.rs`（update_calibre_link_handler追加）
  - `backend/mediavault-api/src/routes/mod.rs`（PATCH /items/:id/files/:file_id/calibre-link登録）
  - `backend/mediavault-api/src/models/item.rs`（ItemDetail.calibre_links拡張）
  - `backend/mediavault-api/src/handlers/items.rs`（get_item_handlerでcalibre_links取得・合成）
- テストファイル: 上記各実装ファイル内の `#[cfg(test)] mod tests`（インライン配置）

## Redフェーズ（失敗するテスト作成）

### 作成日時

2026-06-26

### 方針

note.md・既存実装パターン（QueryBuilder/db_error/ApiError/#[cfg(test)]インライン）を踏襲。
DB非依存ユニットテスト（#[test]）を中心に実装し、DB依存統合テストは#[tokio::test] #[ignore]パターンで追加。
本タスクでは実装自体もRedフェーズ内で完了させ、Greenフェーズ用に「実装済みコード＋未検証（実DB）テスト」という状態で引き渡す
（既存テストが多数#[ignore]のためDocker起動済みDB環境でのみ最終検証可能）。

### 作成したテストケース一覧（22件）

| ID | テスト関数 | 配置ファイル | 種別 |
|---|---|---|---|
| TC-020-U01 | file_not_found_returns_404_with_expected_wire_code | models/response.rs | #[test] |
| TC-020-U02 | update_calibre_link_request_deserializes_valid_fields | models/item_file.rs | #[test] |
| TC-020-U03 | parse_update_calibre_link_request_accepts_non_empty_value | models/item_file.rs | #[test] |
| TC-020-E03 | parse_update_calibre_link_request_rejects_empty_string | models/item_file.rs | #[test] |
| TC-020-E03b | parse_update_calibre_link_request_rejects_whitespace_only | models/item_file.rs | #[test] |
| TC-020-E05 | update_calibre_link_request_with_missing_key_fails_deserialization | models/item_file.rs | #[test] |
| TC-020-E05b | update_calibre_link_request_with_invalid_type_fails_deserialization | models/item_file.rs | #[test] |
| TC-020-B01(unit) | parse_file_id_rejects_invalid_uuid_string | models/item_file.rs | #[test] |
| TC-020-B02b(unit) | parse_file_id_accepts_valid_uuid_string | models/item_file.rs | #[test] |
| TC-020-R01 | update_calibre_link_updates_pdf_record_and_returns_updated_row | repositories/item_file_repository.rs | #[tokio::test] #[ignore] |
| TC-020-R02 | update_calibre_link_with_nonexistent_combination_returns_file_not_found | repositories/item_file_repository.rs | #[tokio::test] #[ignore] |
| TC-020-R03 | update_calibre_link_with_image_file_type_returns_validation_error_without_update | repositories/item_file_repository.rs | #[tokio::test] #[ignore] |
| TC-020-01 | patch_calibre_link_updates_pdf_record_and_returns_200 | handlers/item_files.rs | #[tokio::test] #[ignore] |
| TC-020-N01 | patch_calibre_link_twice_with_same_value_is_idempotent | handlers/item_files.rs | #[tokio::test] #[ignore] |
| TC-020-N02 | patch_calibre_link_overwrites_existing_value | handlers/item_files.rs | #[tokio::test] #[ignore] |
| TC-020-E01 | patch_calibre_link_with_image_file_type_returns_400_and_does_not_update | handlers/item_files.rs | #[tokio::test] #[ignore] |
| TC-020-E02a | patch_calibre_link_with_nonexistent_file_id_returns_404 | handlers/item_files.rs | #[tokio::test] #[ignore] |
| TC-020-E02b | patch_calibre_link_with_mismatched_item_id_returns_404 | handlers/item_files.rs | #[tokio::test] #[ignore] |
| TC-020-E03(router) | patch_calibre_link_with_empty_calibre_book_id_returns_400 | handlers/item_files.rs | #[tokio::test] #[ignore] |
| TC-020-E05(router) | patch_calibre_link_with_missing_key_returns_400 | handlers/item_files.rs | #[tokio::test] #[ignore] |
| TC-020-B01 | patch_calibre_link_with_invalid_item_id_uuid_returns_400 | handlers/item_files.rs | #[tokio::test] #[ignore] |
| TC-020-B02 | patch_calibre_link_with_invalid_file_id_uuid_returns_400 | handlers/item_files.rs | #[tokio::test] #[ignore] |
| TC-020-B03 | patch_calibre_link_with_single_character_returns_200 | handlers/item_files.rs | #[tokio::test] #[ignore] |
| TC-020-B04 | patch_calibre_link_with_surrounding_whitespace_returns_200 | handlers/item_files.rs | #[tokio::test] #[ignore] |
| TC-020-02 | get_item_handler_includes_calibre_web_link_info_for_linked_pdf | handlers/items.rs | #[tokio::test] #[ignore] |
| TC-020-N03 | get_item_handler_does_not_include_calibre_link_for_unlinked_pdf | handlers/items.rs | #[tokio::test] #[ignore] |
| TC-020-B05 | get_item_handler_applies_calibre_link_condition_per_row_with_mixed_files | handlers/items.rs | #[tokio::test] #[ignore] |

合計27件（テストケース定義書の21件 + B02b/E03b/E05bの単体補完バリエーション3件 + E03/E05のrouter統合版2件 + B01 unit版1件）。

### 実装内容（Redフェーズ内で先行実装）

- `ApiErrorCode::FileNotFound`（404・"FILE_NOT_FOUND"）をresponse.rsへ追加
- `UpdateCalibreLinkRequest` DTO・`parse_update_calibre_link_request`・`parse_file_id`・
  `CalibreWebLinkInfo`をitem_file.rsへ追加
- `item_file_repository::update_calibre_link`（2段階判定: 取得→不存在404→file_type≠pdf:400→UPDATE）と
  `get_item_calibre_links`（詳細API用）をitem_file_repository.rsへ追加
- `update_calibre_link_handler`をitem_files.rsへ追加
- `PATCH /items/:id/files/:file_id/calibre-link`ルートをroutes/mod.rsへ登録
- `ItemDetail`に`calibre_links: Vec<CalibreWebLinkInfo>`フィールドを追加し、
  `from_parts_with_calibre_links`コンストラクタを新設（既存`from_parts`は空配列を渡す後方互換ラッパーとして維持）
- `get_item_handler`で`item_file_repository::get_item_calibre_links`を呼び出し、結果を合成

### テスト実行結果

`cargo test -p mediavault-api`実行結果:
- 164 passed; 0 failed; 154 ignored
- 新規追加のDB非依存ユニットテスト8件はすべて`ok`
- 新規追加のDB依存統合テスト18件（リポジトリ3・ハンドラ12・items.rs 3）はすべて`#[ignore]`として正しく登録されコンパイル成功

### 次のフェーズへの要求事項

- 本タスクではRedフェーズ内で実装をほぼ完了させたため、Greenフェーズでは
  `docker compose up -d db`でDBを起動し、`cargo test -p mediavault-api -- --ignored`を実行して
  実際にテストがpassすることを確認するのが主作業となる
- 未確定事項（要件定義書の🔴/🟡部分）:
  - TC-020-B04（trim保存方針）: 本実装ではcalibre_book_idを**原文保持**（trimしない）で保存している。
    Green/Refactorフェーズで方針確定が必要なら見直す
  - Calibre-Web遷移URL構築方式: 本実装では`CalibreWebLinkInfo{file_id, calibre_book_id}`のみを
    保持する最小構造とした。URL自体は含めていない（要件定義書3 L97の「変更容易な構造」方針に従う）
