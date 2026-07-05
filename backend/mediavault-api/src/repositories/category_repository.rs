//! カテゴリ・item_categories のDB操作
//!
//! TASK-0015: タグ・カテゴリCRUD実装（tag_repository.rsと完全に対称な構造）

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::category::{Category, CategoryWithCount};
use crate::models::response::{ApiError, ApiErrorCode};
use crate::repositories::db_error_utils::{is_foreign_key_violation, is_unique_violation};

/// 【機能概要】: sqlxのDBエラーを統一エラー型（INTERNAL_ERROR）へ変換する
/// 【改善内容】: SQLSTATE判定ロジックはdb_error_utilsへ抽出済み（tag_repository::db_errorと対称）
/// 🟡 信頼性レベル: tag_repository::db_errorと完全に対称
fn db_error(err: sqlx::Error) -> ApiError {
    tracing::error!("categories repository db error: {err}");
    ApiError::new(
        ApiErrorCode::InternalError,
        "カテゴリの登録処理に失敗しました",
    )
}

/// 【機能概要】: 新規カテゴリをcategoriesテーブルへINSERTする
/// 【テスト対応】: create_category_inserts_and_returns_category,
/// create_category_with_duplicate_name_returns_conflict_error を通すための実装
/// 🔵 信頼性レベル: タスク仕様テストケース4に直接対応
pub async fn create_category(pool: &PgPool, name: String) -> Result<Category, ApiError> {
    let result: Result<Category, sqlx::Error> =
        sqlx::query_as("INSERT INTO categories (name) VALUES ($1) RETURNING id, name")
            .bind(&name)
            .fetch_one(pool)
            .await;

    result.map_err(|err| {
        if is_unique_violation(&err) {
            ApiError::new(
                ApiErrorCode::DuplicateCategoryName,
                "カテゴリ名は既に存在します",
            )
        } else {
            db_error(err)
        }
    })
}

/// 【機能概要】: カテゴリをcategoriesテーブルから削除する。関連するitem_categoriesはON DELETE CASCADEで自動削除される
/// 【テスト対応】: delete_nonexistent_category_returns_not_found を通すための実装
/// 🔵 信頼性レベル: tag_repository::delete_tagと完全に対称
pub async fn delete_category(pool: &PgPool, id: Uuid) -> Result<bool, ApiError> {
    let result = sqlx::query("DELETE FROM categories WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(db_error)?;

    Ok(result.rows_affected() > 0)
}

/// 【機能概要】: item_categoriesへ複合キー（item_id, category_id）のレコードをINSERTする
/// 【テスト対応】: attach_and_detach_category_to_item を通すための実装
/// 🟡 信頼性レベル: tag_repository::attach_tag_to_itemと完全に対称
pub async fn attach_category_to_item(
    pool: &PgPool,
    item_id: Uuid,
    category_id: Uuid,
) -> Result<(), ApiError> {
    sqlx::query("INSERT INTO item_categories (item_id, category_id) VALUES ($1, $2)")
        .bind(item_id)
        .bind(category_id)
        .execute(pool)
        .await
        .map_err(|err| {
            if is_foreign_key_violation(&err) {
                ApiError::new(
                    ApiErrorCode::CategoryNotFound,
                    "指定されたアイテムまたはカテゴリが見つかりません",
                )
            } else if is_unique_violation(&err) {
                ApiError::new(
                    ApiErrorCode::DuplicateCategoryName,
                    "このカテゴリは既にアイテムへ付与されています",
                )
            } else {
                db_error(err)
            }
        })?;

    Ok(())
}

/// 【機能概要】: item_categoriesから複合キー（item_id, category_id）のレコードをDELETEする
/// 【テスト対応】: attach_and_detach_category_to_item を通すための実装
/// 🟡 信頼性レベル: tag_repository::detach_tag_from_itemと完全に対称
pub async fn detach_category_from_item(
    pool: &PgPool,
    item_id: Uuid,
    category_id: Uuid,
) -> Result<bool, ApiError> {
    let result = sqlx::query("DELETE FROM item_categories WHERE item_id = $1 AND category_id = $2")
        .bind(item_id)
        .bind(category_id)
        .execute(pool)
        .await
        .map_err(db_error)?;

    Ok(result.rows_affected() > 0)
}

/// 【機能概要】: 全カテゴリを、item_categories経由の付与件数（item_count）付きで一覧取得する
/// 🟡 信頼性レベル: tag_repository::list_tags_with_countsと完全に対称
pub async fn list_categories_with_counts(
    pool: &PgPool,
) -> Result<Vec<CategoryWithCount>, ApiError> {
    let categories: Vec<CategoryWithCount> = sqlx::query_as(
        "SELECT c.id, c.name, COUNT(ic.item_id) AS item_count \
        FROM categories c LEFT JOIN item_categories ic ON ic.category_id = c.id \
        GROUP BY c.id, c.name ORDER BY c.name",
    )
    .fetch_all(pool)
    .await
    .map_err(db_error)?;
    Ok(categories)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use uuid::Uuid;

    /// 【テスト用ヘルパー】: docker-compose のテスト用Postgres（DATABASE_URL環境変数）に接続する
    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("TASK-0015統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    /// TC-2（リポジトリ層）: カテゴリ作成がDBへINSERTされ、UUID付きのCategoryを返す
    /// 【テスト目的】: create_category()が新規UUIDを付与してcategoriesテーブルへ正しくINSERTすることを確認する
    /// 【テスト内容】: 実DBに対してcreate_category(pool, "本棚A".to_string())を呼び出す
    /// 【期待される動作】: 返り値のCategory.nameが"本棚A"である
    /// 🔵 信頼性レベル: タスク仕様テストケース4（TASK-0015.md L74-77）と完全一致
    #[tokio::test]
    #[ignore]
    async fn create_category_inserts_and_returns_category() {
        // 【テストデータ準備】: 実DBコネクションプールを取得する
        // 【初期条件設定】: create_categoryはまだ未実装のため、この呼び出し自体がコンパイルエラーとなる想定
        let pool = test_pool().await;

        // 【実際の処理実行】: create_categoryを呼び出す
        // 【処理内容】: INSERT INTO categories (name) VALUES ($1) RETURNING ...を想定
        let category = create_category(&pool, "本棚A".to_string()).await.unwrap();

        // 【結果検証】: nameが入力値と一致することを確認
        assert_eq!(category.name, "本棚A"); // 【確認内容】: 作成されたカテゴリのnameが入力値と一致することを確認 🔵
    }

    /// TC-8: カテゴリ名重複でエラー
    /// 【エラーケースの概要】: 既存カテゴリと同名のnameでcreate_categoryを呼んだ場合の挙動を確認する
    /// 【テスト内容】: 同名カテゴリを2回作成し、2回目がUNIQUE制約違反としてエラーになることを検証する
    /// 【期待される動作】: 2回目の呼び出しがErr(ApiError)となる
    /// 🟡 信頼性レベル: TASK-0015.md 注意事項（L94）の記載に基づく推測
    #[tokio::test]
    #[ignore]
    async fn create_category_with_duplicate_name_returns_conflict_error() {
        // 【テストデータ準備】: 同名のカテゴリ作成を2回試みる
        // 【初期条件設定】: create_categoryはまだ未実装のため、この呼び出し自体がコンパイルエラーとなる想定
        let pool = test_pool().await;
        let name = format!("重複カテゴリ-{}", Uuid::new_v4());

        // 【実際の処理実行】: 1回目は成功、2回目はUNIQUE制約違反(23505)を期待する
        create_category(&pool, name.clone()).await.unwrap();
        let result = create_category(&pool, name).await;

        // 【結果検証】: 2回目がエラーになることを確認
        assert!(result.is_err()); // 【確認内容】: カテゴリ名の一意制約違反が適切にエラーへ変換されることを確認 🟡
    }

    /// TC-10: 存在しないカテゴリIDでDELETE時に404相当
    /// 【エラーケースの概要】: 存在しないUUIDを指定してdelete_categoryを呼んだ場合の挙動を確認する
    /// 【テスト内容】: ランダムなUUIDでdelete_categoryを呼び出し、falseが返ることを検証する
    /// 🟡 信頼性レベル: TC-9（タグ）と対称な推測
    #[tokio::test]
    #[ignore]
    async fn delete_nonexistent_category_returns_not_found() {
        // 【テストデータ準備】: DBに存在しないランダムなUUIDを用意する
        // 【初期条件設定】: delete_categoryはまだ未実装のため、この呼び出し自体がコンパイルエラーとなる想定
        let pool = test_pool().await;
        let nonexistent_id = Uuid::new_v4();

        // 【実際の処理実行】: delete_categoryを呼び出す
        let deleted = delete_category(&pool, nonexistent_id).await.unwrap();

        // 【結果検証】: 削除対象が存在しなかったことを示すfalseが返ることを確認
        assert!(!deleted); // 【確認内容】: 存在しないカテゴリIDの削除がfalse（=404相当）として判定されることを確認 🟡
    }

    /// TC-5: カテゴリのアイテム付与・解除
    /// 【テスト目的】: attach_category_to_item / detach_category_from_itemがタグと同様に動作することを確認する
    /// 【テスト内容】: 既存item, 既存categoryの組み合わせで付与→解除を順に実行する
    /// 【期待される動作】: 付与はOk(())、解除はtrueを返す
    /// 🟡 信頼性レベル: タスク仕様の完了条件「item_categoriesへの付与・削除エンドポイントが実装される」（TASK-0015.md L30）からの推測
    #[tokio::test]
    #[ignore]
    async fn attach_and_detach_category_to_item() {
        // 【テストデータ準備】: 実DBに既存のitem_id, category_idを用意する
        // 【初期条件設定】: attach_category_to_item/detach_category_from_itemはまだ未実装のため、この呼び出し自体がコンパイルエラーとなる想定
        let pool = test_pool().await;
        let item_id = Uuid::new_v4();
        let category_id = Uuid::new_v4();

        // 【実際の処理実行】: 付与→解除の順に呼び出す
        // 【処理内容】: item_categoriesへの複合キーINSERT→DELETEを想定
        attach_category_to_item(&pool, item_id, category_id)
            .await
            .unwrap();
        let detached = detach_category_from_item(&pool, item_id, category_id)
            .await
            .unwrap();

        // 【結果検証】: 解除が成功したことを示すtrueが返ることを確認
        assert!(detached); // 【確認内容】: 付与済みレコードの解除がtrueとして判定されることを確認 🟡
    }
}
