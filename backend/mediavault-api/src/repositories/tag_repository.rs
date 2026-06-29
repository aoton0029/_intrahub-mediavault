//! タグ・item_tags のDB操作
//!
//! TASK-0015: タグ・カテゴリCRUD実装

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::response::{ApiError, ApiErrorCode};
use crate::models::tag::Tag;
use crate::repositories::db_error_utils::{is_foreign_key_violation, is_unique_violation};

/// 【機能概要】: sqlxのDBエラーを統一エラー型（INTERNAL_ERROR）へ変換する
/// 【改善内容】: SQLSTATE判定ロジックはdb_error_utilsへ抽出済み。本関数はtags固有の
/// エラーメッセージ「タグの登録処理に失敗しました」の組み立てにのみ責任を持つ
/// 【設計方針】: item_repository.rsのdb_errorと同一パターンを踏襲し、内部エラー詳細は
/// ログのみに留め、クライアントへは汎用メッセージのみ返す（DB内部情報の漏洩防止）
/// 🟡 信頼性レベル: item_repository.rsのdb_errorパターンを踏襲
fn db_error(err: sqlx::Error) -> ApiError {
    // 【詳細ログ出力】: 内部調査用に詳細をサーバーログへ記録する（クライアントへは返さない） 🟡
    tracing::error!("tags repository db error: {err}");
    ApiError::new(ApiErrorCode::InternalError, "タグの登録処理に失敗しました")
}

/// 【機能概要】: 新規タグをtagsテーブルへINSERTする
/// 【実装方針】: 一意制約違反(23505)は専用のDuplicateTagNameエラー（409）へ変換し、
/// それ以外のDBエラーはdb_errorでINTERNAL_ERRORへ変換する
/// 【テスト対応】: create_tag_inserts_and_returns_tag,
/// create_tag_with_duplicate_name_returns_conflict_error,
/// create_tag_db_error_maps_to_internal_error を通すための実装
/// 🔵 信頼性レベル: タスク仕様テストケース1・2に直接対応
pub async fn create_tag(pool: &PgPool, name: String) -> Result<Tag, ApiError> {
    // 【INSERT実行】: gen_random_uuid()でid自動生成、nameのみ指定してRETURNINGで取得する 🔵
    let result: Result<Tag, sqlx::Error> =
        sqlx::query_as("INSERT INTO tags (name) VALUES ($1) RETURNING id, name")
            .bind(&name)
            .fetch_one(pool)
            .await;

    result.map_err(|err| {
        // 【一意制約違反の個別ハンドリング】: 重複タグ名はDUPLICATE_TAG_NAMEとして409で返す 🟡
        if is_unique_violation(&err) {
            ApiError::new(ApiErrorCode::DuplicateTagName, "タグ名は既に存在します")
        } else {
            db_error(err)
        }
    })
}

/// 【機能概要】: タグをtagsテーブルから削除する。関連するitem_tagsはDBのON DELETE CASCADEで自動削除される
/// 【実装方針】: 影響行数が0の場合はfalseを返し、呼び出し側（ハンドラ）で404判定する
/// 【テスト対応】: delete_nonexistent_tag_returns_not_found, delete_tag_cascades_item_tags を通すための実装
/// 🔵 信頼性レベル: 既存delete_item_handlerの影響行数チェックパターンに対応
pub async fn delete_tag(pool: &PgPool, id: Uuid) -> Result<bool, ApiError> {
    // 【DELETE実行】: ON DELETE CASCADEによりitem_tagsの関連行も同時に削除される 🔵
    let result = sqlx::query("DELETE FROM tags WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(db_error)?;

    Ok(result.rows_affected() > 0)
}

/// 【機能概要】: item_tagsへ複合キー（item_id, tag_id）のレコードをINSERTし、タグをアイテムへ付与する
/// 【実装方針】: 外部キー制約違反(23503)はTagNotFound（存在しないitem/tag）、
/// 一意制約違反(23505)はDuplicateTagName（重複付与）として個別にエラー変換する
/// 【テスト対応】: attach_tag_to_item_inserts_item_tags_row,
/// attach_tag_to_nonexistent_item_returns_not_found,
/// attach_already_attached_tag_returns_conflict_or_noop を通すための実装
/// 🟡 信頼性レベル: タスク仕様「item_tagsへの複合キーINSERT」とDBスキーマFK/PK制約からの妥当な推測
pub async fn attach_tag_to_item(
    pool: &PgPool,
    item_id: Uuid,
    tag_id: Uuid,
) -> Result<(), ApiError> {
    // 【複合キーINSERT】: item_tags(item_id, tag_id)へ付与レコードを作成する 🟡
    sqlx::query("INSERT INTO item_tags (item_id, tag_id) VALUES ($1, $2)")
        .bind(item_id)
        .bind(tag_id)
        .execute(pool)
        .await
        .map_err(|err| {
            if is_foreign_key_violation(&err) {
                ApiError::new(
                    ApiErrorCode::TagNotFound,
                    "指定されたアイテムまたはタグが見つかりません",
                )
            } else if is_unique_violation(&err) {
                ApiError::new(
                    ApiErrorCode::DuplicateTagName,
                    "このタグは既にアイテムへ付与されています",
                )
            } else {
                db_error(err)
            }
        })?;

    Ok(())
}

/// 【機能概要】: item_tagsから複合キー（item_id, tag_id）のレコードをDELETEし、タグの付与を解除する
/// 【実装方針】: 影響行数が0の場合はfalseを返し、呼び出し側（ハンドラ）で404判定する
/// 【テスト対応】: detach_tag_from_item_deletes_item_tags_row を通すための実装
/// 🟡 信頼性レベル: タスク仕様「複合キーでのDELETE」に対応
pub async fn detach_tag_from_item(
    pool: &PgPool,
    item_id: Uuid,
    tag_id: Uuid,
) -> Result<bool, ApiError> {
    // 【複合キーDELETE】: item_tags(item_id, tag_id)の該当行を削除する 🟡
    let result = sqlx::query("DELETE FROM item_tags WHERE item_id = $1 AND tag_id = $2")
        .bind(item_id)
        .bind(tag_id)
        .execute(pool)
        .await
        .map_err(db_error)?;

    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use uuid::Uuid;

    /// 【テスト用ヘルパー】: docker-compose のテスト用Postgres（DATABASE_URL環境変数）に接続する
    /// item_repository.rsのtest_pool()と同一パターン。Greenフェーズで実装する。
    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("TASK-0015統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    /// 【テスト用ヘルパー】: 到達不能なPgPoolを構築し、db_errorのINTERNAL_ERROR変換を検証するために使う
    async fn unreachable_pool() -> PgPool {
        PgPool::connect("postgres://invalid:invalid@127.0.0.1:1/invalid")
            .await
            .expect("到達不能プールの構築検証用接続に失敗しました")
    }

    /// TC-1（リポジトリ層）: タグ作成がDBへINSERTされ、UUID付きのTagを返す
    /// 【テスト目的】: create_tag()が新規UUIDを付与してtagsテーブルへ正しくINSERTすることを確認する
    /// 【テスト内容】: 実DBに対してcreate_tag(pool, "お気に入り".to_string())を呼び出す
    /// 【期待される動作】: 返り値のTag.nameが"お気に入り"、Tag.idがUUID形式である
    /// 🔵 信頼性レベル: タスク仕様テストケース1（TASK-0015.md L59-62）と完全一致
    #[tokio::test]
    #[ignore]
    async fn create_tag_inserts_and_returns_tag() {
        // 【テストデータ準備】: 実DBコネクションプールを取得する
        // 【初期条件設定】: create_tagはまだ未実装のため、この呼び出し自体がコンパイルエラーとなる想定
        let pool = test_pool().await;

        // 【実際の処理実行】: create_tagを呼び出す
        // 【処理内容】: INSERT INTO tags (name) VALUES ($1) RETURNING ...を想定
        let tag = create_tag(&pool, "お気に入り".to_string()).await.unwrap();

        // 【結果検証】: nameが入力値と一致し、idがUUIDであることを確認
        assert_eq!(tag.name, "お気に入り"); // 【確認内容】: 作成されたタグのnameが入力値と一致することを確認 🔵
    }

    /// TC-7: タグ名重複でエラー
    /// 【エラーケースの概要】: 既存タグと同名のnameでcreate_tagを呼んだ場合の挙動を確認する
    /// 【テスト内容】: 同名タグを2回作成し、2回目がUNIQUE制約違反としてエラーになることを検証する
    /// 【期待される動作】: 2回目の呼び出しがErr(ApiError)となり、UnprocessableEntityまたは専用コードを返す
    /// 🟡 信頼性レベル: タスク仕様テストケース2（TASK-0015.md L64-67、エラーコード例「409 DUPLICATE_TAG_NAME」明記）に基づくが最終コードは実装判断
    #[tokio::test]
    #[ignore]
    async fn create_tag_with_duplicate_name_returns_conflict_error() {
        // 【テストデータ準備】: 同名のタグ作成を2回試みる
        // 【初期条件設定】: create_tagはまだ未実装のため、この呼び出し自体がコンパイルエラーとなる想定
        let pool = test_pool().await;
        let name = format!("重複タグ-{}", Uuid::new_v4());

        // 【実際の処理実行】: 1回目は成功、2回目はUNIQUE制約違反(23505)を期待する
        // 【処理内容】: db_errorを通す前にユニーク制約違反を個別検出することを期待する
        create_tag(&pool, name.clone()).await.unwrap();
        let result = create_tag(&pool, name).await;

        // 【結果検証】: 2回目がエラーになることを確認
        assert!(result.is_err()); // 【確認内容】: タグ名の一意制約違反が適切にエラーへ変換されることを確認 🟡
    }

    /// TC-9: 存在しないタグIDでDELETE時に404相当
    /// 【エラーケースの概要】: 存在しないUUIDを指定してdelete_tagを呼んだ場合の挙動を確認する
    /// 【テスト内容】: ランダムなUUIDでdelete_tagを呼び出し、falseまたはNotFoundエラーを検証する
    /// 【期待される動作】: 削除対象が存在しないことを示す結果が返る
    /// 🟡 信頼性レベル: 既存delete_item_handlerの404パターン（影響行数チェック）からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn delete_nonexistent_tag_returns_not_found() {
        // 【テストデータ準備】: DBに存在しないランダムなUUIDを用意する
        // 【初期条件設定】: delete_tagはまだ未実装のため、この呼び出し自体がコンパイルエラーとなる想定
        let pool = test_pool().await;
        let nonexistent_id = Uuid::new_v4();

        // 【実際の処理実行】: delete_tagを呼び出す
        // 【処理内容】: DELETE文の影響行数が0であることを判定する想定
        let deleted = delete_tag(&pool, nonexistent_id).await.unwrap();

        // 【結果検証】: 削除対象が存在しなかったことを示すfalseが返ることを確認
        assert!(!deleted); // 【確認内容】: 存在しないタグIDの削除がfalse（=404相当）として判定されることを確認 🟡
    }

    /// TC-3: タグのアイテム付与
    /// 【テスト目的】: attach_tag_to_itemがitem_tagsテーブルへ複合キーレコードを作成することを確認する
    /// 【テスト内容】: 既存item, 既存tagの組み合わせでattach_tag_to_itemを呼び出す
    /// 【期待される動作】: Ok(())が返り、item_tagsに該当行が存在する
    /// 🟡 信頼性レベル: タスク仕様テストケース3が前提だが、エンドポイント形式・戻り値は推測
    #[tokio::test]
    #[ignore]
    async fn attach_tag_to_item_inserts_item_tags_row() {
        // 【テストデータ準備】: 実DBに既存のitem_id, tag_idを用意する（シードヘルパーはGreenフェーズで整備）
        // 【初期条件設定】: attach_tag_to_itemはまだ未実装のため、この呼び出し自体がコンパイルエラーとなる想定
        let pool = test_pool().await;
        let item_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();

        // 【実際の処理実行】: attach_tag_to_itemを呼び出す
        // 【処理内容】: INSERT INTO item_tags (item_id, tag_id) VALUES ($1, $2)を想定
        let result = attach_tag_to_item(&pool, item_id, tag_id).await;

        // 【結果検証】: 付与処理が成功することを確認
        assert!(result.is_ok()); // 【確認内容】: item_tagsへの複合キーINSERTが成功することを確認 🟡
    }

    /// TC-4: タグのアイテム解除
    /// 【テスト目的】: detach_tag_from_itemがitem_tagsから複合キーで該当行を削除することを確認する
    /// 【テスト内容】: 付与済みのitem_id, tag_idに対してdetach_tag_from_itemを呼び出す
    /// 【期待される動作】: trueが返り、item_tagsから該当行が消える
    /// 🟡 信頼性レベル: タスク仕様テストケース3が前提だが、削除成功時の戻り値表現は推測
    #[tokio::test]
    #[ignore]
    async fn detach_tag_from_item_deletes_item_tags_row() {
        // 【テストデータ準備】: 事前に付与済みのitem_id, tag_idを用意する
        // 【初期条件設定】: detach_tag_from_itemはまだ未実装のため、この呼び出し自体がコンパイルエラーとなる想定
        let pool = test_pool().await;
        let item_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        attach_tag_to_item(&pool, item_id, tag_id).await.unwrap();

        // 【実際の処理実行】: detach_tag_from_itemを呼び出す
        // 【処理内容】: DELETE FROM item_tags WHERE item_id=$1 AND tag_id=$2を想定
        let detached = detach_tag_from_item(&pool, item_id, tag_id).await.unwrap();

        // 【結果検証】: 削除が成功したことを示すtrueが返ることを確認
        assert!(detached); // 【確認内容】: 付与済みレコードの解除がtrueとして判定されることを確認 🟡
    }

    /// TC-6: タグ削除時のカスケード削除確認
    /// 【テスト目的】: delete_tag実行後、ON DELETE CASCADEによりitem_tagsの関連行が自動削除されることを確認する
    /// 【テスト内容】: タグ作成→アイテム付与→タグ削除の順に実DBへ操作し、item_tagsの残存有無を検証する
    /// 【期待される動作】: タグ削除後、該当tag_idのitem_tags行が0件になる
    /// 🟡 信頼性レベル: タスク仕様の統合テスト要件（TASK-0015.md L81）に明記
    #[tokio::test]
    #[ignore]
    async fn delete_tag_cascades_item_tags() {
        // 【テストデータ準備】: タグ作成→アイテム付与の順に実DBへ操作する
        // 【初期条件設定】: create_tag/attach_tag_to_item/delete_tagはまだ未実装のため、この呼び出し自体がコンパイルエラーとなる想定
        let pool = test_pool().await;
        let item_id = Uuid::new_v4();
        let tag = create_tag(&pool, format!("カスケード確認-{}", Uuid::new_v4()))
            .await
            .unwrap();
        attach_tag_to_item(&pool, item_id, tag.id).await.unwrap();

        // 【実際の処理実行】: delete_tagでタグ本体を削除する
        // 【処理内容】: DBのON DELETE CASCADEにより、item_tagsの関連行も自動削除される想定
        delete_tag(&pool, tag.id).await.unwrap();

        // 【結果検証】: item_tagsへ該当tag_idの行が残っていないことを確認する
        let remaining: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM item_tags WHERE tag_id = $1")
            .bind(tag.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(remaining, 0); // 【確認内容】: タグ削除に伴うカスケード削除がDB制約レベルで機能していることを確認 🟡
    }

    /// TC-13: 存在しないitem/tagの組み合わせでの付与時エラー
    /// 【エラーケースの概要】: 存在しないitem_idまたはtag_idを指定して付与しようとした場合の挙動を確認する
    /// 【テスト内容】: 存在しないUUIDでattach_tag_to_itemを呼び出し、エラーになることを検証する
    /// 【期待される動作】: FK制約違反がApiErrorへ変換され、Errが返る
    /// 🟡 信頼性レベル: DBスキーマのFK制約構造からの妥当な推測
    #[tokio::test]
    #[ignore]
    async fn attach_tag_to_nonexistent_item_returns_not_found() {
        // 【テストデータ準備】: DBに存在しないitem_id, tag_idのペアを用意する
        // 【初期条件設定】: attach_tag_to_itemはまだ未実装のため、この呼び出し自体がコンパイルエラーとなる想定
        let pool = test_pool().await;
        let nonexistent_item_id = Uuid::new_v4();
        let nonexistent_tag_id = Uuid::new_v4();

        // 【実際の処理実行】: attach_tag_to_itemを呼び出す
        // 【処理内容】: FK制約違反(23503)がApiErrorへ変換されることを期待する
        let result = attach_tag_to_item(&pool, nonexistent_item_id, nonexistent_tag_id).await;

        // 【結果検証】: 参照整合性違反がエラーとして返ることを確認
        assert!(result.is_err()); // 【確認内容】: 存在しないitem/tagへの付与がFK制約違反としてエラーになることを確認 🟡
    }

    /// TC-14: 既に付与済みのタグを再度付与した場合
    /// 【エラーケースの概要】: 既にitem_tagsに存在する組み合わせへの再付与時の挙動を確認する
    /// 【テスト内容】: 1回目の付与成功後、同じ組み合わせで再度attach_tag_to_itemを呼び出す
    /// 【期待される動作】: 2回目の呼び出しがErr（複合PK制約違反由来）となる
    /// 🟡 信頼性レベル: タスク仕様L45「既存の場合はno-opまたは409」に基づくが最終選択は実装判断
    #[tokio::test]
    #[ignore]
    async fn attach_already_attached_tag_returns_conflict_or_noop() {
        // 【テストデータ準備】: 1回目の付与を実行しておく
        // 【初期条件設定】: attach_tag_to_itemはまだ未実装のため、この呼び出し自体がコンパイルエラーとなる想定
        let pool = test_pool().await;
        let item_id = Uuid::new_v4();
        let tag_id = Uuid::new_v4();
        attach_tag_to_item(&pool, item_id, tag_id).await.unwrap();

        // 【実際の処理実行】: 同じ組み合わせで再度attach_tag_to_itemを呼び出す
        // 【処理内容】: 複合PK制約違反(23505)が発生する想定
        let result = attach_tag_to_item(&pool, item_id, tag_id).await;

        // 【結果検証】: 重複付与がエラーとして返ることを確認（本実装方針: no-opではなくエラーとする）
        assert!(result.is_err()); // 【確認内容】: 重複付与が複合PK制約違反としてエラーになることを確認 🟡
    }

    /// DBエラー変換テスト: db_errorがINTERNAL_ERROR/500相当に変換されることを確認する
    /// 【テスト目的】: 到達不能なPgPoolに対するcreate_tag呼び出しがInternalErrorへ変換されることを確認する
    /// 【テスト内容】: unreachable_pool()を使ってcreate_tagを呼び出し、エラーになることを検証する
    /// 【期待される動作】: Err(ApiError { ... InternalError ... })相当が返る
    /// 🟡 信頼性レベル: item_repository.rsのdb_error変換テストパターンと同型の推測
    #[tokio::test]
    #[ignore]
    async fn create_tag_db_error_maps_to_internal_error() {
        // 【テストデータ準備】: 到達不能なPgPoolを構築する
        // 【初期条件設定】: db_error/create_tagはまだ未実装のため、この呼び出し自体がコンパイルエラーとなる想定
        let pool = unreachable_pool().await;

        // 【実際の処理実行】: create_tagを呼び出す
        // 【処理内容】: 接続不能によるsqlx::Errorがdb_errorで変換されることを期待する
        let result = create_tag(&pool, "テスト".to_string()).await;

        // 【結果検証】: エラーが返ることを確認
        assert!(result.is_err()); // 【確認内容】: DB接続不能時にエラーが適切にApiErrorへ変換されることを確認 🟡
    }
}
