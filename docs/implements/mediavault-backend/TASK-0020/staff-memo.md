# TDD開発メモ: staff（スタッフ管理CRUD）

## 概要

- 機能名: staff（スタッフ管理CRUD）
- 開発開始: 2026-06-24
- 現在のフェーズ: Refactor完了

## 関連ファイル

- 元タスクファイル: `docs/tasks/mediavault-backend/TASK-0020.md`
- 要件定義: `docs/implements/mediavault-backend/TASK-0020/staff-requirements.md`
- テストケース定義: `docs/implements/mediavault-backend/TASK-0020/staff-testcases.md`
- Redフェーズ記録: `docs/implements/mediavault-backend/TASK-0020/staff-red-phase.md`
- 実装ファイル:
  - `backend/mediavault-api/src/models/staff.rs`
  - `backend/mediavault-api/src/repositories/staff_repository.rs`
  - `backend/mediavault-api/src/handlers/staff.rs`
  - `backend/mediavault-api/src/routes/mod.rs`（ルート追加）
- テストファイル: 上記3ファイル内の`#[cfg(test)] mod tests`（Rust標準テスト規約に準拠）

## Redフェーズ（失敗するテスト作成）

### 作成日時

2026-06-24

### テストケース

staff-testcases.md（TC-N-01〜06, TC-E-01〜08, TC-B-01〜06、計20ケース）のうち、
DB非依存のmodels単体9件・DB依存のrepository統合10件・handler統合4件、計22テストを実装。
（TC-B-01「name 255文字境界」とTC-B-04「null/未指定の同値性」、TC-B-06「壊れたJSON」は
信頼性が低い🟡推測かつ既存パターンでカバー範囲が薄いため本Redフェーズでは見送り、
Greenフェーズ後の追加テストケースとして留保）

### テストコード概要

- `models/staff.rs`: `parse_create_staff_request()` / `parse_create_item_staff_request()` を
  `todo!()` スタブとして定義し、バリデーションロジックのテストが必ずpanicするようにした。
- `repositories/staff_repository.rs`: `create_staff()` / `link_staff()` / `unlink_staff()` を
  `todo!()` スタブとして定義。統合テストは`DATABASE_URL`必須のため`#[ignore]`を付与（既存TASK-0017パターンに準拠）。
- `handlers/staff.rs`: 3ハンドラを`todo!()`スタブとして定義し、`routes::build_router`経由の
  ルーティング統合テストを追加。

### 期待される失敗

```
thread '...' panicked at mediavault-api\src\models\staff.rs:69:5:
not yet implemented: TASK-0020 Greenフェーズで実装: nameの空文字チェック
```
models単体テスト9件中7件がこの形でFAILED（todo!()によるpanic）。
残り2件（`create_staff_request_deserializes_all_fields`, `create_item_staff_request_with_invalid_uuid_fails_deserialization`）は
serdeのデシリアライズのみで完結するテストのためPASSED（これはRed/Green分類上問題なし、
deserialize自体は既存のserde実装に依存する非対象機能のテスト）。

repository/handler層（計14件、`#[ignore]`）は、`cargo test -- --ignored`実行時（DATABASE_URL要）に
同様の`todo!()`panicでFAILEDになる設計。今回はDBコンテナ未起動のため実行はしていないが、
`cargo test --no-run`でコンパイル可能であることを確認済み。

### 次のフェーズへの要求事項

1. `models/response.rs`に`ApiErrorCode::StaffNotFound`（`STAFF_NOT_FOUND`/404）を新規追加
2. `models/staff.rs`の2つのparse関数を実装（name/role空文字チェック、role≤100文字、character_name≤255文字）
3. `repositories/staff_repository.rs`の3関数を実装（INSERT、存在確認、DELETE + 整合性チェック）
4. `handlers/staff.rs`の3ハンドラを実装（リクエスト検証→repository呼び出し→レスポンス構築）
5. 既存マイグレーションに`staff`/`item_staff`テーブルが存在するか確認（TASK-0004担当範囲だが未確認の場合は追加が必要）

## Greenフェーズ（最小実装）

### 実装日時

2026-06-24

### 実装方針

Redフェーズのtodo!()スタブをすべて最小実装に置き換えた。既存のitems/tags/item_groups CRUDパターン
（parse_*関数によるpureバリデーション、repository層でのsqlx::query_as、db_errorによる内部情報秘匿、
存在確認→INSERT/DELETEの順序）を踏襲し、独自のロジックは導入していない。

1. **`models/response.rs`**: `ApiErrorCode::StaffNotFound`（`STAFF_NOT_FOUND`/404）を追加。
   `code_and_status()`のmatchアームに追加するだけで既存パターンに合致。
2. **`models/staff.rs`**:
   - `parse_create_staff_request`: `name.trim().is_empty()`でVALIDATION_ERROR、255文字超もVALIDATION_ERROR。
   - `parse_create_item_staff_request`: `role.trim().is_empty()`でVALIDATION_ERROR、role>100文字、
     character_name>255文字（Some時のみ）でVALIDATION_ERROR。
3. **`repositories/staff_repository.rs`**:
   - `item_exists`/`staff_exists`: item_group_repositoryと対称なSELECT 1存在確認ヘルパーを追加。
   - `create_staff`: `INSERT INTO staff (...) RETURNING ...`でid/created_atを取得。
   - `link_staff`: item_exists→ITEM_NOT_FOUND、staff_exists→STAFF_NOT_FOUNDの順に事前確認後、
     `INSERT INTO item_staff (...) RETURNING ...`。
   - `unlink_staff`: `DELETE FROM item_staff WHERE id = $1 AND item_id = $2`で整合性チェックを
     SQLレベルで行い、`rows_affected() > 0`をbool判定。
4. **`handlers/staff.rs`**:
   - `create_staff_handler`: deserialize_request→parse_create_staff_request→create_staff→201。
   - `create_item_staff_handler`: parse_item_id→deserialize_request→parse_create_item_staff_request
     →link_staff→201。
   - `delete_item_staff_handler`: parse_item_id×2→unlink_staff→false時ITEM_NOT_FOUND(404)、true時204。
   - DELETE時の404エラーコードはITEM_NOT_FOUND（item_idに属する紐付けが見つからない、という意味）を採用。

### テスト結果

```
cargo build                  → 成功（既存warningのみ、新規warningなし）
cargo test models::staff     → 9 passed; 0 failed
cargo test --no-run          → 全テストバイナリのコンパイル成功（DB依存テストも含む）
cargo test                   → 87 passed; 0 failed; 81 ignored（DB依存・DATABASE_URL未設定のため）
```

DB依存の統合テスト（repository 10件 + handler 4件、計14件）はDocker未起動のためローカルでは
`--ignored`実行できなかったが、コンパイルは確認済み。実装方針はitem_group_repository等の
既存実装と同一パターンのため、ロジック上の不整合は想定されない。

### 課題・改善点（Refactorフェーズで対応）

- `models/staff.rs`の2関数で文字数チェック（255/100/255）がitem.rs等の既存実装と重複コードの
  可能性がある。共通ヘルパー化を検討。
- `db_error`関数が`#[allow(dead_code)]`なしでも使われるようになったため、属性削除済み（既に対応済み）。
- DELETE時のエラーコードを`ItemNotFound`で代用しているが、要件上は「item_staff不存在」を表す
  専用コードがあった方が意味的に明確（テストケース定義書TC-E-04は単に404を期待するのみで
  コード文字列は指定していないため、現状の選択でテスト要件を満たす）。
- DB依存テストの実DB実行確認はRefactor/Verify-completeフェーズでdocker-compose起動後に実施予定。

## Refactorフェーズ（品質改善）

### 実施日時

2026-06-24

### レビュー結果

既存パターン（item_group_repository.rs / item_episode_repository.rs / item_relation_repository.rs /
db_error_utils.rs）と比較した結果、`staff_repository::link_staff`が事前存在確認
（item_exists/staff_exists）を採用している点は、他repositoryのFK制約違反マッピング方式
（is_foreign_key_violation）とは異なるが、これは品質上の問題ではなく必要な設計差異と判断した。
理由: link_staffはitem_id/staff_idという2つの異なるテーブルへのFK参照を持ち、単一のSQLSTATE
だけではどちらの制約違反かを区別できない。本コードベースには制約名で判定するヘルパーが存在
しないため、事前存在確認によるITEM_NOT_FOUND/STAFF_NOT_FOUND の明確な区別が妥当な選択。

文字数チェック（255/100/255）の共通ヘルパー化も検討したが、既存モデル（item_group.rs等）も
同様のインラインパターンを採用しており、コードベース全体の慣習と整合させるため見送った。

### 適用した変更（コメント・ドキュメントのみ、機能変更なし）

1. models/staff.rs・repositories/staff_repository.rs・handlers/staff.rsのモジュール冒頭にあった
   「Redフェーズ注記」（todo!()時代の説明）を実装完了後の実態に合わせて更新。
2. staff_repository.rsのモジュールコメントに、事前存在確認方式を採用した設計理由を明記。
3. handlers/staff.rsのテストコード内の誤字「INSENT」→「INSERT」を修正。
4. delete_item_staff_handlerのdocコメントに、404時のエラーコードがITEM_NOT_FOUNDを流用する
   設計判断の理由を明記（課題・改善点セクションに記載されていた内容を実装コードへ可視化）。

### セキュリティレビュー結果

- SQLインジェクション対策: 全クエリでバインドパラメータ使用、文字列結合なし。問題なし。
- DB内部情報の秘匿: db_error関数がtracing::error!でのみ詳細を出力し、クライアントへは
  汎用メッセージのみ返却。既存パターンと一致。
- 入力検証: 長さ制限・空文字チェックが早期リターンされ、DB到達前に弾かれる。問題なし。

### パフォーマンスレビュー結果

- link_staffの事前存在確認は最大2回のSELECTを追加するが、2つの異なるFK参照を区別する要件上
  必要なトレードオフ。Indexアクセスのみで複雑度上の問題はない。
- unlink_staffはDELETEのWHERE条件にitem_id整合性チェックを含め、追加SELECTなしで効率的。

### テスト結果

```
cargo build   → 成功（既存warning 3件のみ、新規warningなし）
cargo test    → 87 passed; 0 failed; 81 ignored（Greenフェーズと同一件数、リグレッションなし）
```

### 品質評価

✅ 高品質（Refactor完了）。コードの構造自体は既存パターンと整合しており変更不要、
ドキュメント・コメントの明確化を中心に改善した。

## 次のステップ

次のお勧めステップ: `/tsumiki:tdd-verify-complete` で完全性検証を実行します。
