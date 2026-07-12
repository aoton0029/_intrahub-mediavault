//! staff / item_staff のDB操作
//!
//! TASK-0020: スタッフ管理CRUD実装（item_relation_repository.rsと対称な構造）
//!
//! 【実装状況】: Greenフェーズで全関数を実装済み。
//! 【設計方針】: link_staffはitem_id/staff_idという2つの異なるテーブルへのFK参照を持つため、
//! 単一のFK制約違反検出（db_error_utils::is_foreign_key_violation）では「どちらが原因か」を
//! 区別できない（このコードベースには制約名で判定するヘルパーが存在しないため）。
//! そのためitem_exists/staff_existsによる事前存在確認で、ITEM_NOT_FOUND/STAFF_NOT_FOUNDを
//! 明確に区別する設計を採用している（item_relation_repository等のFK制約違反マッピングとは
//! 異なるが、エラーコードの一意性を優先した意図的な選択）。
//! 統合テストは実DB接続（DATABASE_URL）が必要なため#[ignore]を付与している。

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::response::{ApiError, ApiErrorCode};
use crate::models::staff::{ItemStaff, ItemStaffWithName, Staff, StaffSearchResult};

/// 【機能概要】: sqlxのDBエラーを統一エラー型（INTERNAL_ERROR）へ変換する
/// 🔵 信頼性レベル: item_relation_repository::db_errorと対称
fn db_error(err: sqlx::Error) -> ApiError {
    tracing::error!("staff repository db error: {err}");
    ApiError::new(
        ApiErrorCode::InternalError,
        "スタッフの登録処理に失敗しました",
    )
}

/// 【機能概要】: 指定したitem_idがitemsテーブルに存在するか確認する
/// 【実装方針】: item_group_repository::item_existsと対称な存在確認ヘルパー
/// 🔵 信頼性レベル: staff-requirements.md 2.2「item_idの存在を事前確認する」に対応
pub async fn item_exists(pool: &PgPool, item_id: Uuid) -> Result<bool, ApiError> {
    let result: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM items WHERE id = $1")
        .bind(item_id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    Ok(result.is_some())
}

/// 【機能概要】: 指定したstaff_idがstaffテーブルに存在するか確認する
/// 【実装方針】: item_existsと対称な存在確認ヘルパー
/// 🔵 信頼性レベル: staff-requirements.md 2.2「staff_idの存在を事前確認する」に対応
pub async fn staff_exists(pool: &PgPool, staff_id: Uuid) -> Result<bool, ApiError> {
    let result: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM staff WHERE id = $1")
        .bind(staff_id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    Ok(result.is_some())
}

/// 【機能概要】: 氏名部分一致（ILIKE）でstaffを検索し、紐付け作品数（linked_item_count）を併記して返す
/// 【実装方針】: item_staffをLEFT JOINしてCOUNTし、name ILIKE検索でヒットしたスタッフをname昇順・limit件返す
pub async fn search_staff(
    pool: &PgPool,
    query: &str,
    limit: i64,
) -> Result<Vec<StaffSearchResult>, ApiError> {
    sqlx::query_as::<_, StaffSearchResult>(
        "SELECT s.id, s.external_id, s.name, s.image_url, s.created_at,
                COUNT(i.id) AS linked_item_count
         FROM staff s
         LEFT JOIN item_staff i ON i.staff_id = s.id
         WHERE s.name ILIKE $1
         GROUP BY s.id
         ORDER BY s.name
         LIMIT $2",
    )
    .bind(format!("%{query}%"))
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(db_error)
}

/// 【機能概要】: staffテーブルへ新規スタッフレコードをINSERTする
/// 【実装方針】: name/external_id/image_urlを受け取りINSERT、idとcreated_atはDB側で自動生成
/// 【テスト対応】: TC-N-01, TC-N-02 を通すための実装
/// 🔵 信頼性レベル: TASK-0020 完了条件1・テストケース1 に直接対応
pub async fn create_staff(
    pool: &PgPool,
    name: String,
    external_id: Option<String>,
    image_url: Option<String>,
) -> Result<Staff, ApiError> {
    // 【メイン処理】: staffテーブルへINSERTし、生成されたid/created_atを含めて返す 🔵
    sqlx::query_as::<_, Staff>(
        "INSERT INTO staff (external_id, name, image_url)
         VALUES ($1, $2, $3)
         RETURNING id, external_id, name, image_url, created_at",
    )
    .bind(&external_id)
    .bind(&name)
    .bind(&image_url)
    .fetch_one(pool)
    .await
    .map_err(db_error)
}

/// 【機能概要】: external_id一致の既存staffを返すか、無ければ新規作成する
/// 【実装方針】: インポート連携（Annict等）でのスタッフ重複登録を避けるため、
/// external_id（"annict-person-<id>"等の一意キー）で既存行を検索し、あればそれを再利用する
pub async fn find_or_create_staff_by_external_id(
    pool: &PgPool,
    external_id: Option<String>,
    name: String,
    image_url: Option<String>,
) -> Result<Staff, ApiError> {
    if let Some(ext_id) = &external_id {
        let existing: Option<Staff> = sqlx::query_as(
            "SELECT id, external_id, name, image_url, created_at FROM staff WHERE external_id = $1",
        )
        .bind(ext_id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

        if let Some(staff) = existing {
            return Ok(staff);
        }
    }

    create_staff(pool, name, external_id, image_url).await
}

/// 【機能概要】: item_id/staff_idの存在を確認し、item_staffへ新規紐付けレコードをINSERTする
/// 【実装方針】: staff_id不存在はSTAFF_NOT_FOUND、item_id不存在はITEM_NOT_FOUNDへマッピングする。
/// アプリ側で事前存在確認を行うことで、FK制約違反より詳細なエラーコードを返す（staff-requirements.md 3）。
/// 【テスト対応】: TC-N-03, TC-N-04, TC-E-02, TC-E-03 を通すための実装
/// 🔵 信頼性レベル: TASK-0020 完了条件2・3・テストケース2・3・5 に直接対応
pub async fn link_staff(
    pool: &PgPool,
    item_id: Uuid,
    staff_id: Uuid,
    role: String,
    character_name: Option<String>,
) -> Result<ItemStaff, ApiError> {
    // 【入力値検証】: item_idの存在を事前確認する（不存在ならITEM_NOT_FOUND） 🟡
    if !item_exists(pool, item_id).await? {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定されたアイテムが見つかりません",
        ));
    }

    // 【入力値検証】: staff_idの存在を事前確認する（不存在ならSTAFF_NOT_FOUND） 🔵
    if !staff_exists(pool, staff_id).await? {
        return Err(ApiError::new(
            ApiErrorCode::StaffNotFound,
            "指定されたスタッフが見つかりません",
        ));
    }

    // 【メイン処理】: item_staffへINSERTし、生成されたidを含めて返す 🔵
    sqlx::query_as::<_, ItemStaff>(
        "INSERT INTO item_staff (item_id, staff_id, role, character_name)
         VALUES ($1, $2, $3, $4)
         RETURNING id, item_id, staff_id, role, character_name",
    )
    .bind(item_id)
    .bind(staff_id)
    .bind(&role)
    .bind(&character_name)
    .fetch_one(pool)
    .await
    .map_err(db_error)
}

/// 【機能概要】: 指定item_idに紐づくスタッフ紐付けを一覧取得する（`GET /items/:id/staff`）
/// 🟡 信頼性レベル: item_relation_repository::list_item_relationsと対称のリスト取得パターン
pub async fn list_item_staff(
    pool: &PgPool,
    item_id: Uuid,
) -> Result<Vec<ItemStaffWithName>, ApiError> {
    sqlx::query_as(
        "SELECT item_staff.id, item_staff.item_id, item_staff.staff_id, item_staff.role,
                item_staff.character_name, staff.name AS staff_name, staff.image_url AS staff_image_url
         FROM item_staff
         JOIN staff ON staff.id = item_staff.staff_id
         WHERE item_staff.item_id = $1",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await
    .map_err(db_error)
}

/// 【機能概要】: item_staffから指定idのレコードをDELETEする（item_idとの整合性チェック含む）
/// 【実装方針】: item_staff.id == item_staff_id AND item_staff.item_id == item_id の条件でDELETEし、
/// 影響行が0件ならfalseを返す（呼び出し側で404にマッピングする）
/// 【テスト対応】: TC-N-05, TC-E-04, TC-E-05 を通すための実装
/// 🟡 信頼性レベル: TASK-0020 完了条件4・5・実装詳細3（item_id整合性チェック）に対応
pub async fn unlink_staff(
    pool: &PgPool,
    item_id: Uuid,
    item_staff_id: Uuid,
) -> Result<bool, ApiError> {
    // 【メイン処理】: item_idとitem_staff_idの両方が一致する行のみDELETEする（整合性チェック） 🟡
    let result = sqlx::query("DELETE FROM item_staff WHERE id = $1 AND item_id = $2")
        .bind(item_staff_id)
        .bind(item_id)
        .execute(pool)
        .await
        .map_err(db_error)?;

    // 【結果返却】: 影響行数が0件ならfalse（呼び出し側ハンドラで404へマッピングする） 🟡
    Ok(result.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    /// 【テスト用ヘルパー】: docker-compose のテスト用Postgres（DATABASE_URL環境変数）に接続する
    /// 🔵 信頼性レベル: item_relation_repository::test_poolと対称
    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("TASK-0020統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    /// TC-N-01: 必須フィールドnameのみでスタッフ作成が成功し201相当のStaffが返る
    /// 🔵 信頼性レベル: TASK-0020 テストケース1 / staff-testcases.md TC-N-01 に明記
    #[tokio::test]
    #[ignore] // 実DB（docker compose up -d db）が必要。cargo test -- --ignored で実行
    async fn create_staff_with_required_fields_only_returns_staff_with_null_optionals() {
        // 【テスト目的】: nameのみでスタッフ作成が成功し、external_id/image_urlがnullで返ることを確認する
        // 【テスト内容】: create_staff(pool, "監督A", None, None) を呼び出す
        // 【期待される動作】: Staffが返り、external_id/image_urlがNone、idがUUID、nameが"監督A"
        // 🔵 信頼性レベル: TASK-0020 テストケース1 に明記

        // 【テストデータ準備】: クリーンなテストDBプールを用意
        // 【初期条件設定】: staffテーブルに当該nameの行が存在しない状態
        let pool = test_pool().await;

        // 【実際の処理実行】: create_staff repository関数を直接呼び出す
        // 【処理内容】: staffテーブルへのINSERTを期待する（未実装のためtodo!()でpanicする）
        let result = create_staff(&pool, "監督A".to_string(), None, None).await;

        // 【結果検証】: Okであり、フィールド値が期待通りであること
        // 【期待値確認】: external_id/image_urlはoptionalなのでNone、nameは入力値と一致
        let staff = result.expect("create_staffは成功するはず"); // 【確認内容】: 作成が成功することを確認 🔵
        assert_eq!(staff.name, "監督A"); // 【確認内容】: name値が保持されることを確認 🔵
        assert!(staff.external_id.is_none()); // 【確認内容】: external_id未指定はNoneであることを確認 🔵
        assert!(staff.image_url.is_none()); // 【確認内容】: image_url未指定はNoneであることを確認 🔵
    }

    /// TC-N-02: name/external_id/image_url全指定でスタッフ作成が成功する
    /// 🔵 信頼性レベル: staff-requirements.md 2.1 / note.md セクション4 入力仕様に基づく
    #[tokio::test]
    #[ignore]
    async fn create_staff_with_all_fields_persists_optional_fields() {
        // 【テスト目的】: optional含む全フィールドが正しく保存されることを確認する
        // 【テスト内容】: create_staff(pool, "声優B", Some("anilist-12345"), Some(url)) を呼び出す
        // 【期待される動作】: external_id/image_urlが欠落せず保存される
        // 🔵 信頼性レベル: staff-requirements.md 2.1 に基づく

        // 【テストデータ準備】: 全フィールド指定のテストデータ
        let pool = test_pool().await;

        // 【実際の処理実行】: create_staff repository関数を直接呼び出す
        let result = create_staff(
            &pool,
            "声優B".to_string(),
            Some("anilist-12345".to_string()),
            Some("https://example.com/b.png".to_string()),
        )
        .await;

        // 【結果検証】: 全フィールドが入力値と一致すること
        let staff = result.expect("create_staffは成功するはず"); // 【確認内容】: 作成が成功することを確認 🔵
        assert_eq!(staff.external_id, Some("anilist-12345".to_string())); // 【確認内容】: external_idが保持されることを確認 🔵
        assert_eq!(
            staff.image_url,
            Some("https://example.com/b.png".to_string())
        ); // 【確認内容】: image_urlが保持されることを確認 🔵
    }

    /// TC-N-03: 既存item・既存staffに対しrole指定で紐付けが成功する（character_nameなし）
    /// 🔵 信頼性レベル: TASK-0020 テストケース2 / staff-testcases.md TC-N-03 に明記
    #[tokio::test]
    #[ignore]
    async fn link_staff_without_character_name_creates_item_staff_record() {
        // 【テスト目的】: item_id/staff_idの存在確認後、item_staffへINSERTされ201相当のItemStaffが返ることを確認する
        // 【テスト内容】: link_staff(pool, item_id, staff_id, "監督", None) を呼び出す
        // 【期待される動作】: ItemStaffが返り、character_nameがNone
        // 🔵 信頼性レベル: TASK-0020 テストケース2 に明記

        // 【テストデータ準備】: 仮のitem_id/staff_id（実際の統合テストでは事前にitem/staffを作成する）
        let pool = test_pool().await;
        let item_id = Uuid::new_v4();
        let staff_id = Uuid::new_v4();

        // 【実際の処理実行】: link_staff repository関数を直接呼び出す
        let result = link_staff(&pool, item_id, staff_id, "監督".to_string(), None).await;

        // 【結果検証】: Okであり、character_nameがNoneであること
        let item_staff = result.expect("link_staffは成功するはず"); // 【確認内容】: 紐付け作成が成功することを確認 🔵
        assert_eq!(item_staff.role, "監督"); // 【確認内容】: role値が保持されることを確認 🔵
        assert!(item_staff.character_name.is_none()); // 【確認内容】: character_name未指定はNoneであることを確認 🔵
    }

    /// TC-N-04: character_name付きの紐付け（声優役）が正しく保存される
    /// 🟡 信頼性レベル: TASK-0020 テストケース3（🟡） / staff-testcases.md TC-N-04
    #[tokio::test]
    #[ignore]
    async fn link_staff_with_character_name_persists_character_name() {
        // 【テスト目的】: character_nameが正しく保存されることを確認する
        // 【テスト内容】: link_staff(pool, item_id, staff_id, "声優", Some("主人公")) を呼び出す
        // 【期待される動作】: ItemStaff.character_nameが"主人公"
        // 🟡 信頼性レベル: TASK-0020 テストケース3（🟡）

        // 【テストデータ準備】: 仮のitem_id/staff_id
        let pool = test_pool().await;
        let item_id = Uuid::new_v4();
        let staff_id = Uuid::new_v4();

        // 【実際の処理実行】: link_staff repository関数を直接呼び出す
        let result = link_staff(
            &pool,
            item_id,
            staff_id,
            "声優".to_string(),
            Some("主人公".to_string()),
        )
        .await;

        // 【結果検証】: character_nameが入力値と一致すること
        let item_staff = result.expect("link_staffは成功するはず"); // 【確認内容】: 紐付け作成が成功することを確認 🟡
        assert_eq!(item_staff.character_name, Some("主人公".to_string())); // 【確認内容】: character_nameが保持されることを確認 🟡
    }

    /// TC-E-02: 存在しないstaff_idでの紐付けはSTAFF_NOT_FOUNDを返す
    /// 🔵 信頼性レベル: TASK-0020 完了条件・テストケース5 / staff-requirements.md 2.2・4.3 に明記
    #[tokio::test]
    #[ignore]
    async fn link_staff_with_nonexistent_staff_id_returns_staff_not_found() {
        // 【テスト目的】: 不存在のstaff_idで紐付けようとした場合にSTAFF_NOT_FOUND(404)が返ることを確認する
        // 【テスト内容】: link_staffに存在しないstaff_idを渡す
        // 【期待される動作】: Err(ApiError { code: "STAFF_NOT_FOUND" }) が返る
        // 🔵 信頼性レベル: TASK-0020 完了条件・テストケース5 に明記

        // 【テストデータ準備】: 既存item_id（仮）と、DBに存在しないstaff_id
        let pool = test_pool().await;
        let item_id = Uuid::new_v4();
        let nonexistent_staff_id = Uuid::new_v4();

        // 【実際の処理実行】: link_staff repository関数を直接呼び出す
        let result = link_staff(
            &pool,
            item_id,
            nonexistent_staff_id,
            "監督".to_string(),
            None,
        )
        .await;

        // 【結果検証】: Errであり、エラーコードがSTAFF_NOT_FOUNDであること
        let err = result.expect_err("不存在staff_idはエラーになるはず"); // 【確認内容】: エラーが返ることを確認 🔵
        assert_eq!(err.error.code, "STAFF_NOT_FOUND"); // 【確認内容】: STAFF_NOT_FOUNDエラーコードが返ることを確認 🔵
    }

    /// TC-E-03: 存在しないitem_idでの紐付けはITEM_NOT_FOUNDを返す
    /// 🟡 信頼性レベル: staff-requirements.md 2.2・4.3（🟡、ITEM_NOT_FOUNDは既存流用）
    #[tokio::test]
    #[ignore]
    async fn link_staff_with_nonexistent_item_id_returns_item_not_found() {
        // 【テスト目的】: 不存在のitem_idで紐付けようとした場合にITEM_NOT_FOUND(404)が返ることを確認する
        // 【テスト内容】: link_staffに存在しないitem_idを渡す
        // 【期待される動作】: Err(ApiError { code: "ITEM_NOT_FOUND" }) が返る
        // 🟡 信頼性レベル: staff-requirements.md 2.2・4.3（🟡）

        // 【テストデータ準備】: DBに存在しないitem_idと、既存staff_id（仮）
        let pool = test_pool().await;
        let nonexistent_item_id = Uuid::new_v4();
        let staff_id = Uuid::new_v4();

        // 【実際の処理実行】: link_staff repository関数を直接呼び出す
        let result = link_staff(
            &pool,
            nonexistent_item_id,
            staff_id,
            "監督".to_string(),
            None,
        )
        .await;

        // 【結果検証】: Errであり、エラーコードがITEM_NOT_FOUNDであること
        let err = result.expect_err("不存在item_idはエラーになるはず"); // 【確認内容】: エラーが返ることを確認 🟡
        assert_eq!(err.error.code, "ITEM_NOT_FOUND"); // 【確認内容】: ITEM_NOT_FOUNDエラーコードが返ることを確認 🟡
    }

    /// TC-N-05: 既存item_staffレコードの削除が成功する
    /// 🟡 信頼性レベル: TASK-0020 テストケース4（🟡） / staff-testcases.md TC-N-05
    #[tokio::test]
    #[ignore]
    async fn unlink_staff_with_existing_record_returns_true() {
        // 【テスト目的】: 整合するitem_id/item_staff_idでの削除が成功しtrueを返すことを確認する
        // 【テスト内容】: unlink_staff(pool, item_id, item_staff_id) を呼び出す
        // 【期待される動作】: Ok(true) が返り、当該レコードがDBから消える
        // 🟡 信頼性レベル: TASK-0020 テストケース4（🟡）

        // 【テストデータ準備】: 仮のitem_id/item_staff_id（実際の統合テストでは事前にlink_staffで作成する）
        let pool = test_pool().await;
        let item_id = Uuid::new_v4();
        let item_staff_id = Uuid::new_v4();

        // 【実際の処理実行】: unlink_staff repository関数を直接呼び出す
        let result = unlink_staff(&pool, item_id, item_staff_id).await;

        // 【結果検証】: Okかつtrueであること
        let deleted = result.expect("unlink_staffは成功するはず"); // 【確認内容】: 削除処理自体が成功することを確認 🟡
        assert!(deleted); // 【確認内容】: 既存レコードの削除でtrueが返ることを確認 🟡
    }

    /// TC-E-04: 存在しないitem_staff_idでの削除はfalseを返す（呼び出し側で404にマッピング）
    /// 🟡 信頼性レベル: TASK-0020 完了条件 / staff-requirements.md 4.3（🟡）
    #[tokio::test]
    #[ignore]
    async fn unlink_staff_with_nonexistent_item_staff_id_returns_false() {
        // 【テスト目的】: 不存在のitem_staff_idでの削除が影響行0件としてfalseを返すことを確認する
        // 【テスト内容】: unlink_staff(pool, item_id, 不存在item_staff_id) を呼び出す
        // 【期待される動作】: Ok(false) が返る（呼び出し側のハンドラで404へマッピングする）
        // 🟡 信頼性レベル: TASK-0020 完了条件 / staff-requirements.md 4.3（🟡）

        // 【テストデータ準備】: 既存item_id（仮）と、DBに存在しないitem_staff_id
        let pool = test_pool().await;
        let item_id = Uuid::new_v4();
        let nonexistent_item_staff_id = Uuid::new_v4();

        // 【実際の処理実行】: unlink_staff repository関数を直接呼び出す
        let result = unlink_staff(&pool, item_id, nonexistent_item_staff_id).await;

        // 【結果検証】: Okかつfalseであること（サイレント成功を避けるためfalseで明示）
        let deleted = result.expect("unlink_staffはエラーにならないはず（false判定はハンドラ側）"); // 【確認内容】: 削除処理自体はエラーにならないことを確認 🟡
        assert!(!deleted); // 【確認内容】: 不存在レコードの削除でfalseが返ることを確認 🟡
    }

    /// TC-E-05: item_idに属さないitem_staff_idでの削除はfalseを返す（整合性チェック）
    /// 🟡 信頼性レベル: TASK-0020 実装詳細3 / staff-requirements.md 4.3（🟡、妥当推測）
    #[tokio::test]
    #[ignore]
    async fn unlink_staff_with_mismatched_item_id_returns_false() {
        // 【テスト目的】: item_staffは存在するが指定item_idと不一致の場合にfalseを返すことを確認する
        // 【テスト内容】: itemBに属するitem_staff_idを、itemAのitem_idで削除しようとする
        // 【期待される動作】: Ok(false) が返り、itemBの紐付けは削除されない（整合性ガード）
        // 🟡 信頼性レベル: TASK-0020 実装詳細3（🟡、妥当推測）

        // 【テストデータ準備】: itemA（指定するitem_id）とitemBに属する仮のitem_staff_id
        let pool = test_pool().await;
        let item_a_id = Uuid::new_v4();
        let item_staff_id_belonging_to_item_b = Uuid::new_v4();

        // 【実際の処理実行】: unlink_staff repository関数を直接呼び出す
        let result = unlink_staff(&pool, item_a_id, item_staff_id_belonging_to_item_b).await;

        // 【結果検証】: Okかつfalseであること（他itemのデータを保護）
        let deleted = result.expect("unlink_staffはエラーにならないはず（false判定はハンドラ側）"); // 【確認内容】: 削除処理自体はエラーにならないことを確認 🟡
        assert!(!deleted); // 【確認内容】: item_id不一致の場合にfalseが返り誤削除を防止することを確認 🟡
    }

    /// TC-B-05: staff削除時に関連item_staffがCASCADE削除される（統合テスト）
    /// 🟡 信頼性レベル: TASK-0020 統合テスト要件 / staff-requirements.md 4.3（🟡）
    #[tokio::test]
    #[ignore]
    async fn deleting_staff_cascades_to_item_staff_records() {
        // 【テスト目的】: 親staff削除時に子item_staffがON DELETE CASCADEで自動削除されることを確認する
        // 【テスト内容】: staff作成 → item_staff紐付け → staffをDBから直接DELETE → item_staffが残っていないことを確認
        // 【期待される動作】: staff削除後、当該staff_idを参照するitem_staffが0件になる
        // 🟡 信頼性レベル: TASK-0020 統合テスト要件（🟡）

        // 【テストデータ準備】: staff作成 → item_staff紐付けの一連のセットアップ
        let pool = test_pool().await;
        let staff = create_staff(&pool, "カスケード確認用".to_string(), None, None)
            .await
            .expect("create_staffは成功するはず");
        let item_id = Uuid::new_v4();
        let _item_staff = link_staff(&pool, item_id, staff.id, "監督".to_string(), None)
            .await
            .expect("link_staffは成功するはず");

        // 【実際の処理実行】: staffテーブルから直接DELETEする（CASCADE発火を期待）
        sqlx::query("DELETE FROM staff WHERE id = $1")
            .bind(staff.id)
            .execute(&pool)
            .await
            .expect("staff削除に失敗");

        // 【結果検証】: item_staffに当該staff_idを参照する行が残っていないこと
        let remaining: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM item_staff WHERE staff_id = $1")
                .bind(staff.id)
                .fetch_one(&pool)
                .await
                .expect("COUNT取得に失敗");
        assert_eq!(remaining, 0); // 【確認内容】: CASCADE削除により孤立した参照行が残らないことを確認 🟡
    }
}
