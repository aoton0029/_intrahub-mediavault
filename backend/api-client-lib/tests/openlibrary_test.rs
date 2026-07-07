use std::future::Future;

use api_client_lib::clients::openlibrary::requests::{
    OlIsbnRequest, OlSearchRequest, OlWorksRequest,
};
use api_client_lib::clients::openlibrary::OpenLibraryClient;
use api_client_lib::error::ApiError;

/// OpenLibrary の実 API は共有インフラ上でのレスポンスタイムが不安定なため、
/// クライアント側の固定 3 秒タイムアウトに対してまれに間に合わないことがある。
/// テストの検証内容（ステータス・レスポンス内容）は変えず、`ApiError::Timeout`
/// による一時的な失敗のみをリトライして安定化する。
async fn with_timeout_retry<T, F, Fut>(attempts: u32, mut f: F) -> Result<T, ApiError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, ApiError>>,
{
    let mut last_err = None;
    for attempt in 0..attempts {
        match f().await {
            Ok(v) => return Ok(v),
            Err(ApiError::Timeout) => {
                last_err = Some(ApiError::Timeout);
                if attempt + 1 < attempts {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                    continue;
                }
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_err.unwrap_or(ApiError::Timeout))
}

#[tokio::test]
async fn search_returns_results() {
    let client = OpenLibraryClient::new().expect("client init");

    let resp = with_timeout_retry(10, || {
        client.search(OlSearchRequest {
            q: "Rust programming language".into(),
            page: None,
            limit: Some(5),
        })
    })
    .await
    .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert!(!resp.model.is_empty());
}

#[tokio::test]
async fn get_by_isbn_returns_edition() {
    let client = OpenLibraryClient::new().expect("client init");

    // 9780134685991: Effective Java (3rd Edition)
    let resp = with_timeout_retry(10, || {
        client.get_by_isbn(OlIsbnRequest {
            isbn: "9780134685991".into(),
        })
    })
    .await
    .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert!(resp.model.title.is_some());
}

#[tokio::test]
async fn get_works_returns_work() {
    let client = OpenLibraryClient::new().expect("client init");

    let resp = with_timeout_retry(10, || {
        client.get_works(OlWorksRequest {
            olid: "OL45804W".into(),
        })
    })
    .await
    .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert!(resp.model.title.is_some());
}
