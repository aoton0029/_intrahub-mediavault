# TASK-0020 Refactorフェーズ記録: スタッフ管理CRUD実装

**タスクID**: TASK-0020
**機能名**: staff（スタッフ管理CRUD）
**要件名**: mediavault-backend
**作成日**: 2026-06-24

---

## 1. レビュー結果

### 1.1 既存パターンとの比較

`item_group_repository.rs` / `item_episode_repository.rs` / `item_relation_repository.rs` /
`db_error_utils.rs` と比較し、以下を確認した。

- **db_errorパターン**: `tracing::error!` + `ApiErrorCode::InternalError`への変換は既存と同一形式。問題なし。
- **FK制約違反のマッピング方式の違い（意図的な設計差異）**:
  既存repository（item_episode/item_relation等）は `db_error_utils::is_foreign_key_violation` /
  `is_unique_violation` でDB制約違反を検出してからエラーコードへマッピングする。
  一方`staff_repository::link_staff`は `item_exists`/`staff_exists` による事前存在確認を採用している。
  **これはコード品質の問題ではなく必要な設計差異**: `link_staff`はitem_id/staff_idという
  2つの異なるテーブルへのFK参照を持ち、単一のSQLSTATE（23503）だけでは「item側」「staff側」
  どちらの制約違反かを区別できない。本コードベースには制約名（constraint name）で判定する
  ヘルパーが存在しないため（`db_error_utils.rs`を確認、`is_foreign_key_violation`はSQLSTATEの
  みで判定）、ITEM_NOT_FOUND/STAFF_NOT_FOUNDを明確に区別するには事前存在確認が妥当な選択である。
  この設計意図をコードコメントとして明記した（後述）。
- **重複コード（DRY）**: `parse_create_staff_request`/`parse_create_item_staff_request`の文字数
  チェックは、`item_group.rs`/`item_episode.rs`等の既存モデルにも同様のインラインパターンが
  あり、フィールド名・エラーメッセージが異なるため共通ヘルパー化のメリットは小さいと判断し、
  現状維持とした（既存コードベース全体の慣習と整合）。
- **命名**: `item_exists`/`staff_exists`、`create_staff`/`link_staff`/`unlink_staff`、
  `created_staff_response`/`created_item_staff_response` は既存の`item_group_repository`/
  `handlers::item_groups`と対称な命名規則に従っており問題なし。
- **エラーハンドリング**: DB内部情報の秘匿（`db_error`関数内で`tracing::error!`のみに出力）も
  既存パターンと一致。

### 1.2 セキュリティレビュー

- SQLインジェクション: 全クエリが`sqlx::query`/`query_as`のバインドパラメータ（`$1`等）を使用、
  文字列結合なし。問題なし。
- DB内部情報の漏洩: `db_error`関数が`sqlx::Error`の詳細をクライアントへ返さず、汎用メッセージ
  のみ返却。サーバーログにのみ`tracing::error!`で出力。既存パターンと一致し問題なし。
- 入力検証: name/role/character_nameの長さ制限・空文字チェックが早期リターンされ、DB到達前に
  弾かれる。UUID形式チェックはserdeデシリアライズ段階で実施（既存パターンと同一）。

### 1.3 パフォーマンスレビュー

- `link_staff`は事前存在確認のため最大2回のSELECT + 1回のINSERTとなり、FK制約違反方式
  （1回のINSERTのみ）より往復が増える。ただし2つの異なるテーブルへのFK参照を区別して
  エラーコードを返す要件上、許容されるコストと判断（item数件レベルの小規模Indexアクセスで
  あり、N+1のような複雑度の問題ではない）。
- `unlink_staff`はDELETE 1回のみでitem_id整合性チェックをSQL条件（WHERE id = $1 AND item_id = $2）
  に含めており、追加のSELECTを発行しない効率的な実装。

## 2. 適用した変更

機能的な変更は行わず、以下のコメント・ドキュメント改善のみを適用した（テスト behavior は不変）。

1. **`backend/mediavault-api/src/models/staff.rs`**: モジュール冒頭の「Redフェーズ注記」
   （todo!()スタブ時代の説明）が実装完了後も残っていたため、Greenフェーズ完了後の実態を表す
   「実装状況」コメントへ更新。
2. **`backend/mediavault-api/src/repositories/staff_repository.rs`**: 同様にモジュール冒頭コメントを
   更新し、`link_staff`が事前存在確認を採用している設計理由（2つの異なるFK参照を区別するため、
   制約名判定ヘルパーが本コードベースに存在しないため）を明記。
3. **`backend/mediavault-api/src/handlers/staff.rs`**:
   - モジュール冒頭コメントを実装完了後の状態に更新。
   - テストコード内の誤字「INSENT」→「INSERT」を修正。
   - `delete_item_staff_handler`のdocコメントに、404時のエラーコードが`ItemNotFound`を流用する
     設計判断の理由（テストケース定義書が文字列を指定していないため専用コード追加を見送った旨）
     を明記し、memo.mdに記録されていた暗黙の判断を可視化した。

🔵 信頼性レベル: いずれもdocs/implements配下の既存記録（staff-green-phase.md, staff-memo.md）
および既存repositoryファイル（item_group/item_episode/item_relation_repository.rs,
db_error_utils.rs）の実装内容と直接照合した結果に基づく。

## 3. テスト実行結果

```bash
cd backend/mediavault-api
cargo build   # 成功（既存warning 3件のみ、新規warningなし）
cargo test    # 87 passed; 0 failed; 81 ignored（Greenフェーズと同一件数）
```

機能的変更を行っていないため、テスト結果はGreenフェーズと完全に同一（87 passed / 0 failed /
81 ignored）。リグレッションなし。

## 4. 品質判定

| 評価項目 | 状態 |
|---|---|
| テスト結果 | ✅ 87 passed / 0 failed / 81 ignored（Greenフェーズと同一） |
| セキュリティ | ✅ 重大な脆弱性なし（SQLインジェクション対策・内部情報秘匿を確認） |
| パフォーマンス | ✅ 重大な性能課題なし（事前存在確認のコストは設計上必要なトレードオフと判断） |
| リファクタ品質 | ✅ 既存パターンとの整合性を確認済み、コード自体は変更不要と判断 |
| コード品質 | ✅ コメント・ドキュメントの明確性を向上 |
| ファイルサイズ | ✅ いずれも500行未満（models/staff.rs 約190行、repositories/staff_repository.rs 約265行、handlers/staff.rs 約225行） |

**総合評価**: ✅ 高品質（Refactor）

実装コード自体（ロジック）は既存のitem_group/item_episode/item_relationパターンと整合しており、
構造的な変更は不要と判断した。今回の改善はドキュメント・コメントレベルの明確化が中心である。

---

## 5. 次のステップ

次のお勧めステップ: `/tsumiki:tdd-verify-complete` で完全性検証を実行します。
