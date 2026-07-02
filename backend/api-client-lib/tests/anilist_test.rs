use std::future::Future;

use api_client_lib::auth::AuthStrategy;
use api_client_lib::clients::anilist::requests::MediaType;
use api_client_lib::clients::anilist::requests::{MediaDetailsRequest, MediaSearchRequest};
use api_client_lib::clients::anilist::AniListClient;
use api_client_lib::error::ApiError;

/// AniList の実 API は共有インフラ上で一時的に不安定になることがあり、
/// タイムアウトや 5xx（Internal Server Error 等）、レート制限がまれに発生する。
/// テストの検証内容（ステータス・レスポンス内容）は変えず、これらの一時的な
/// エラーのみをリトライして安定化する。
fn is_transient(err: &ApiError) -> bool {
    matches!(
        err,
        ApiError::Timeout | ApiError::RateLimit { .. }
    ) || matches!(err, ApiError::Http { status, .. } if *status >= 500 && *status < 600)
}

async fn with_transient_retry<T, F, Fut>(attempts: u32, mut f: F) -> Result<T, ApiError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, ApiError>>,
{
    let mut last_err = None;
    for attempt in 0..attempts {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) if is_transient(&e) => {
                last_err = Some(e);
                if attempt + 1 < attempts {
                    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                    continue;
                }
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_err.unwrap_or(ApiError::Timeout))
}

#[tokio::test]
async fn search_media_returns_results() {
    let client = AniListClient::new(AuthStrategy::None).expect("client init");

    let resp = with_transient_retry(10, || {
        client.search_media(MediaSearchRequest {
            search: Some("Naruto".into()),
            media_type: Some(MediaType::Anime),
            ..Default::default()
        })
    })
    .await
    .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert!(!resp.model.is_empty());
}

#[tokio::test]
async fn get_media_details_returns_media() {
    let client = AniListClient::new(AuthStrategy::None).expect("client init");

    let resp = with_transient_retry(10, || client.get_media_details(MediaDetailsRequest { id: 1 }))
        .await
        .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert_eq!(resp.model.id, 1);
}
