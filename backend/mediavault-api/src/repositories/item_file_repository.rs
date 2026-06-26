//! item_files のDB操作
//!
//! TASK-0026: POST /items/:id/files（パス指定方式）実装（item_link_repository.rsと対称な構造）

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::item_file::{CalibreWebLinkInfo, FileType, ItemFile};
use crate::models::response::{ApiError, ApiErrorCode};

/// 【機能概要】: sqlxのDBエラーを統一エラー型（INTERNAL_ERROR）へ変換する
/// 🟡 信頼性レベル: item_link_repository::db_errorと対称
fn db_error(err: sqlx::Error) -> ApiError {
    tracing::error!("item_files repository db error: {err}");
    ApiError::new(
        ApiErrorCode::InternalError,
        "ファイルの登録処理に失敗しました",
    )
}

/// 【機能概要】: 指定したitem_idがitemsテーブルに存在するか確認する
/// 【実装方針】: TASK-0027のupload_item_file_handlerが書込前のitem存在確認に再利用するためpub化する
/// 🔵 信頼性レベル: item_link_repository::item_existsと対称
pub async fn item_exists(pool: &PgPool, item_id: Uuid) -> Result<bool, ApiError> {
    let result: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM items WHERE id = $1")
        .bind(item_id)
        .fetch_optional(pool)
        .await
        .map_err(db_error)?;

    Ok(result.is_some())
}

/// 【機能概要】: item_idの存在を確認し、item_filesへ新規ファイルレコードをINSERTする（パス指定方式）
/// 【実装方針】: calibre_book_idは本タスクではNULL固定とする（TASK-0028で更新対象）
/// 🔵 信頼性レベル: タスク仕様完了条件「該当itemが存在しない場合、ITEM_NOT_FOUND（404）を返す」に直接対応
pub async fn create_item_file(
    pool: &PgPool,
    item_id: Uuid,
    path: String,
    label: Option<String>,
    file_type: FileType,
) -> Result<ItemFile, ApiError> {
    if !item_exists(pool, item_id).await? {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定されたアイテムが見つかりません",
        ));
    }

    sqlx::query_as::<_, ItemFile>(
        "INSERT INTO item_files (item_id, path, label, file_type, calibre_book_id)
         VALUES ($1, $2, $3, $4, NULL)
         RETURNING id, item_id, path, label, file_type, calibre_book_id, created_at",
    )
    .bind(item_id)
    .bind(&path)
    .bind(&label)
    .bind(file_type)
    .fetch_one(pool)
    .await
    .map_err(db_error)
}

/// 【機能概要】: 対象のitem_files行（id+item_id一致）のcalibre_book_idを更新する。
/// file_type=pdf以外への紐付けは拒否(400)、対象行が0件の場合は404を返す2段階方式を取る。
/// 【実装方針】: (a) id+item_idで対象行を取得 → (b) 0行なら404(FileNotFound) →
/// (c) 取得行のfile_type≠pdfなら400(ValidationError) → (d) pdfならUPDATE ... RETURNING
/// という順序で判定する。単純な`WHERE ... AND file_type='pdf'`のみでは
/// 不存在(404)とfile_type不一致(400)が共に0行となり区別できないため、この2段階方式が必要
/// 🔵 信頼性レベル: calibre-link-requirements.md 第3章 L91-92・calibre-link-testcases.md
/// TC-020-R01/R02/R03・「重要（検証順序）」に直接対応
pub async fn update_calibre_link(
    pool: &PgPool,
    item_id: Uuid,
    file_id: Uuid,
    calibre_book_id: String,
) -> Result<ItemFile, ApiError> {
    // 【対象行取得】: id+item_id一致の行を取得する。0行ならFILE_NOT_FOUND(404) 🔵
    let existing = sqlx::query_as::<_, ItemFile>(
        "SELECT id, item_id, path, label, file_type, calibre_book_id, created_at
         FROM item_files
         WHERE id = $1 AND item_id = $2",
    )
    .bind(file_id)
    .bind(item_id)
    .fetch_optional(pool)
    .await
    .map_err(db_error)?;

    let existing = existing.ok_or_else(|| {
        ApiError::new(ApiErrorCode::FileNotFound, "ファイルが見つかりません")
    })?;

    // 【file_type検証】: pdf以外への紐付けはVALIDATION_ERROR(400)で拒否する。
    // 不存在(404)と区別するため、取得済み行に対してここで判定する 🔵
    if existing.file_type != FileType::Pdf {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "pdf以外のファイルにはcalibre_book_idを紐付けできません",
        ));
    }

    // 【UPDATE実行】: pdfであることが確定した行のcalibre_book_idを更新し、更新後行を返す 🔵
    sqlx::query_as::<_, ItemFile>(
        "UPDATE item_files
         SET calibre_book_id = $1
         WHERE id = $2 AND item_id = $3 AND file_type = 'pdf'
         RETURNING id, item_id, path, label, file_type, calibre_book_id, created_at",
    )
    .bind(&calibre_book_id)
    .bind(file_id)
    .bind(item_id)
    .fetch_one(pool)
    .await
    .map_err(db_error)
}

/// 【機能概要】: item_idに紐づくPDF item_filesのうち、calibre_book_id設定済みのものについて
/// Calibre-Web遷移情報の一覧を取得する
/// 【実装方針】: `WHERE item_id = $1 AND file_type = 'pdf' AND calibre_book_id IS NOT NULL`の
/// 条件でitem_filesを絞り込み、CalibreWebLinkInfoへマッピングする。アイテム詳細API
/// （GET /items/:id）レスポンス拡張のために呼び出される
/// 🟡 信頼性レベル: calibre-link-requirements.md 2.3 L77・calibre-link-testcases.md
/// TC-020-02/N03/B05より新規追加
pub async fn get_item_calibre_links(
    pool: &PgPool,
    item_id: Uuid,
) -> Result<Vec<CalibreWebLinkInfo>, ApiError> {
    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, calibre_book_id FROM item_files
         WHERE item_id = $1 AND file_type = 'pdf' AND calibre_book_id IS NOT NULL",
    )
    .bind(item_id)
    .fetch_all(pool)
    .await
    .map_err(db_error)?;

    Ok(rows
        .into_iter()
        .map(|(file_id, calibre_book_id)| CalibreWebLinkInfo {
            file_id,
            calibre_book_id,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    /// 【テスト用ヘルパー】: docker-compose のテスト用Postgres（DATABASE_URL環境変数）に接続する
    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("TASK-0026統合テストにはDATABASE_URL環境変数が必要です");
        PgPool::connect(&url)
            .await
            .expect("テスト用DBへの接続に失敗しました")
    }

    /// テストケース2: 存在しないitem_idでINSERTするとITEM_NOT_FOUNDになる
    #[tokio::test]
    #[ignore]
    async fn create_item_file_with_nonexistent_item_returns_item_not_found() {
        let pool = test_pool().await;

        let result = create_item_file(
            &pool,
            Uuid::new_v4(),
            "/srv/files/pdf/example.pdf".to_string(),
            Some("本編PDF".to_string()),
            FileType::Pdf,
        )
        .await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "ITEM_NOT_FOUND");
    }

    /// 【テスト用ヘルパー】: テスト用itemをDBへ作成しitem_idを返す
    /// 【Green修正】: media_type enumの実際の値は
    /// {anime,movie,drama,manga,novel,game,academic_book,paper}であり'book'は無効値のため
    /// 'novel'に修正。さらにitems.sourceはNOT NULL制約のためデフォルト省略では失敗することが
    /// 判明したため、既存handlers::items.rsのinsert_test_item_for_handlerパターンに倣い明示的に
    /// 'manual'を指定する 🔵
    async fn insert_test_item(db: &PgPool) -> Uuid {
        sqlx::query_scalar(
            "INSERT INTO items (media_type, title, status, is_favorite, source, external_id) \
             VALUES ('novel', 'TASK-0028テスト書籍', 'not_started', false, 'manual', NULL) \
             RETURNING id",
        )
        .fetch_one(db)
        .await
        .expect("テスト用item作成に失敗しました")
    }

    /// 【テスト用ヘルパー】: 指定file_typeのitem_file行を作成しfile_idを返す
    async fn insert_test_item_file(db: &PgPool, item_id: Uuid, file_type: FileType) -> Uuid {
        sqlx::query_scalar(
            "INSERT INTO item_files (item_id, path, label, file_type, calibre_book_id)
             VALUES ($1, '/srv/files/pdf/example.pdf', '本編PDF', $2, NULL)
             RETURNING id",
        )
        .bind(item_id)
        .bind(file_type)
        .fetch_one(db)
        .await
        .expect("テスト用item_file作成に失敗しました")
    }

    /// TC-020-R01: update_calibre_linkがfile_type=pdfのレコードのcalibre_book_idを更新し更新後行を返す
    /// 【テスト目的】: 実DBでfile_type=pdfのitem_files行に対しupdate_calibre_linkを呼ぶと
    /// calibre_book_idが更新され、更新後のItemFileが返ることを確認する
    /// 【テスト内容】: pdf行を事前insertし、update_calibre_linkを呼び出してRETURNING結果とDB再取得値を検証する
    /// 【期待される動作】: Ok(ItemFile{ calibre_book_id: Some("calibre-12345"), file_type: Pdf, .. })、
    /// SELECTで再確認しても更新済み
    /// 🔵 信頼性レベル: タスク定義書 L51-56・要件定義書2.3 L75・TC-020-01に直接対応
    /// 【Red期待】: update_calibre_linkが未実装のためコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn update_calibre_link_updates_pdf_record_and_returns_updated_row() {
        let pool = test_pool().await;
        let item_id = insert_test_item(&pool).await;
        let file_id = insert_test_item_file(&pool, item_id, FileType::Pdf).await;

        let result = update_calibre_link(
            &pool,
            item_id,
            file_id,
            "calibre-12345".to_string(),
        )
        .await;

        let file = result.expect("pdf行の更新は成功するはず");
        assert_eq!(file.calibre_book_id, Some("calibre-12345".to_string())); // 【確認内容】: RETURNINGの値がcalibre-12345に更新されていることを確認 🔵
        assert_eq!(file.file_type, FileType::Pdf); // 【確認内容】: file_typeがpdfのまま変更されていないことを確認 🔵

        let persisted: Option<String> = sqlx::query_scalar(
            "SELECT calibre_book_id FROM item_files WHERE id = $1",
        )
        .bind(file_id)
        .fetch_one(&pool)
        .await
        .expect("再取得に失敗しました");
        assert_eq!(persisted, Some("calibre-12345".to_string())); // 【確認内容】: DB上の値も永続的に更新されていることを確認 🔵
    }

    /// TC-020-R02: update_calibre_linkが不存在の組み合わせでFileNotFound(404相当)を返す
    /// 【テスト目的】: 対象行が存在しない場合にハンドラの404判定の基礎となる戻り値を確認する
    /// 【テスト内容】: ランダムなitem_id/file_id（DBに存在しない）でupdate_calibre_linkを呼ぶ
    /// 【期待される動作】: Err(ApiError{ code: FILE_NOT_FOUND })、UPDATEは実行されない
    /// 🟡 信頼性レベル: 要件定義書 第3章 L91-92（fetch_optionalで0行→None、ハンドラで404）からの妥当な推測
    /// 【Red期待】: update_calibre_linkが未実装のためコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn update_calibre_link_with_nonexistent_combination_returns_file_not_found() {
        let pool = test_pool().await;

        let result = update_calibre_link(
            &pool,
            Uuid::new_v4(),
            Uuid::new_v4(),
            "calibre-12345".to_string(),
        )
        .await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "FILE_NOT_FOUND"); // 【確認内容】: 不存在の組み合わせでFILE_NOT_FOUNDが返ることを確認 🟡
    }

    /// TC-020-R03: update_calibre_linkが存在するimage行に対しValidationError相当を返しUPDATEしない
    /// 【テスト目的】: 行は存在するがfile_type=imageの場合、不存在(404)ではなく
    /// 「存在するがpdfでない(400)」を正しく区別することを確認する（2段階判定の中核）
    /// 【テスト内容】: file_type=image行を事前insertし、update_calibre_linkを呼んで
    /// エラーコードとDB未更新を検証する
    /// 【期待される動作】: Err(ApiError{ code: VALIDATION_ERROR })、該当行のcalibre_book_idは未更新（NULLのまま）
    /// 🔵 信頼性レベル: 要件定義書 第3章 L92（2段階方式）・タスク定義書 テストケース2に直接対応
    /// 【Red期待】: update_calibre_linkが未実装のためコンパイルエラーとなる想定
    #[tokio::test]
    #[ignore]
    async fn update_calibre_link_with_image_file_type_returns_validation_error_without_update() {
        let pool = test_pool().await;
        let item_id = insert_test_item(&pool).await;
        let file_id = insert_test_item_file(&pool, item_id, FileType::Image).await;

        let result = update_calibre_link(
            &pool,
            item_id,
            file_id,
            "calibre-12345".to_string(),
        )
        .await;

        let err = result.unwrap_err();
        assert_eq!(err.error.code, "VALIDATION_ERROR"); // 【確認内容】: 存在するがpdfでない行はVALIDATION_ERROR(400)になることを確認 🔵

        let persisted: Option<String> = sqlx::query_scalar(
            "SELECT calibre_book_id FROM item_files WHERE id = $1",
        )
        .bind(file_id)
        .fetch_one(&pool)
        .await
        .expect("再取得に失敗しました");
        assert_eq!(persisted, None); // 【確認内容】: 拒否時にDBが変更されていない（calibre_book_idがNULLのまま）ことを確認 🔵
    }
}
