//! 抽出済み本文を固定チャンクで返す公開APIハンドラ。

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;
use uuid::Uuid;

use crate::AppState;
use crate::models::item::parse_item_id;
use crate::models::item_file_text::{
    AmbiguousFileCandidate, ItemTextQuery, ItemTextResponse, TextBoundary, TextChunk,
    ValidatedItemTextQuery, compose_chunk_label, parse_item_text_query,
};
use crate::models::response::{ApiError, ApiErrorCode, ApiOk};
use crate::repositories::item_file_text_repository::{self, PrimaryFileResolution};
use crate::repositories::{item_file_repository, item_repository};

#[derive(Debug, Serialize)]
struct AmbiguousFileErrorBody {
    code: &'static str,
    message: &'static str,
    candidates: Vec<AmbiguousFileCandidate>,
}

#[derive(Debug, Serialize)]
struct AmbiguousFileError {
    success: bool,
    error: AmbiguousFileErrorBody,
}

/// 🔵 Intent: 抽出状態には依存せず、保存済み本文だけをDB側でチャンク化して返す。
pub async fn get_item_text_handler(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    Query(query): Query<ItemTextQuery>,
) -> Result<axum::response::Response, ApiError> {
    let item_id = parse_item_id(&item_id)?;
    if item_repository::get_item_by_id(&state.db, item_id)
        .await?
        .is_none()
    {
        return Err(ApiError::new(
            ApiErrorCode::ItemNotFound,
            "指定されたアイテムが見つかりません",
        ));
    }

    let query = parse_item_text_query(query)?;
    let file_id = match resolve_target_file(&state, item_id, &query).await? {
        TargetFile::Resolved(file_id) => file_id,
        TargetFile::Ambiguous(candidates) => return Ok(ambiguous_response(candidates)),
    };

    let row = item_file_text_repository::fetch_chunk(
        &state.db,
        query.chunk_index,
        query.chunk_size,
        file_id,
    )
    .await?
    .ok_or_else(text_not_extracted)?;

    // 空の保存済み本文は有効な抽出結果としてindex=0の空チャンクを返す。
    if chunk_index_out_of_range(query.chunk_index, row.total_chunks) {
        return Err(ApiError::new(
            ApiErrorCode::ValidationError,
            "chunk_indexがチャンク数の範囲外です",
        ));
    }

    let boundaries: Vec<TextBoundary> =
        serde_json::from_value(row.boundaries).map_err(|error| {
            tracing::error!(%error, "item_file_texts boundaries deserialization failed");
            ApiError::new(
                ApiErrorCode::InternalError,
                "抽出テキストの取得処理に失敗しました",
            )
        })?;
    let chunk_start = query.chunk_index * query.chunk_size;
    let chunk_end = chunk_end(chunk_start, &row.chunk_text);
    let label = compose_chunk_label(&boundaries, chunk_start, chunk_end);

    Ok(ApiOk::new(ItemTextResponse {
        item_id,
        file_id,
        extracted_at: row.extracted_at,
        extraction_version: row.extraction_version,
        chunk: TextChunk {
            index: query.chunk_index,
            size: query.chunk_size,
            total_chunks: row.total_chunks,
            label,
            text: row.chunk_text,
        },
    })
    .into_response())
}

enum TargetFile {
    Resolved(Uuid),
    Ambiguous(Vec<AmbiguousFileCandidate>),
}

async fn resolve_target_file(
    state: &AppState,
    item_id: Uuid,
    query: &ValidatedItemTextQuery,
) -> Result<TargetFile, ApiError> {
    if let Some(file_id) = query.file_id {
        let file = item_file_repository::find_item_file(&state.db, item_id, file_id).await?;
        if file.is_none() {
            return Err(file_not_found());
        }
        if !item_file_text_repository::text_exists(&state.db, file_id).await? {
            return Err(text_not_extracted());
        }
        return Ok(TargetFile::Resolved(file_id));
    }

    match item_file_text_repository::resolve_primary_file(&state.db, item_id).await? {
        PrimaryFileResolution::Single(file_id) => Ok(TargetFile::Resolved(file_id)),
        PrimaryFileResolution::NoFiles => Err(file_not_found()),
        PrimaryFileResolution::NoneExtracted => Err(text_not_extracted()),
        PrimaryFileResolution::Ambiguous(candidates) => Ok(TargetFile::Ambiguous(candidates)),
    }
}

fn chunk_end(chunk_start: i64, text: &str) -> i64 {
    chunk_start + text.chars().count() as i64
}

fn chunk_index_out_of_range(chunk_index: i64, total_chunks: i64) -> bool {
    (total_chunks == 0 && chunk_index > 0) || (total_chunks > 0 && chunk_index >= total_chunks)
}

fn file_not_found() -> ApiError {
    ApiError::new(
        ApiErrorCode::FileNotFound,
        "指定されたアイテムにはファイルがありません",
    )
}

fn text_not_extracted() -> ApiError {
    ApiError::new(
        ApiErrorCode::TextNotExtracted,
        "指定されたファイルのテキストはまだ抽出されていません",
    )
}

fn ambiguous_response(candidates: Vec<AmbiguousFileCandidate>) -> axum::response::Response {
    (
        StatusCode::CONFLICT,
        Json(AmbiguousFileError {
            success: false,
            error: AmbiguousFileErrorBody {
                code: "AMBIGUOUS_FILE",
                message: "複数のファイルが抽出済みです。file_idを指定してください",
                candidates,
            },
        }),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_end_uses_actual_character_count() {
        assert_eq!(chunk_end(4_000, &"日".repeat(500)), 4_500);
    }

    #[test]
    fn empty_text_allows_only_the_default_zero_index() {
        assert!(!chunk_index_out_of_range(0, 0));
        assert!(chunk_index_out_of_range(1, 0));
    }
}
