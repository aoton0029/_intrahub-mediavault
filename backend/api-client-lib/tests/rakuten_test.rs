use api_client_lib::clients::rakuten::requests::SearchBooksRequest;
use api_client_lib::clients::rakuten::RakutenClient;
use api_client_lib::AuthStrategy;

fn rakuten_auth() -> Option<AuthStrategy> {
    let application_id = std::env::var("RAKUTEN_APPLICATION_ID").ok()?;
    let access_key = std::env::var("RAKUTEN_ACCESS_KEY").ok()?;
    if application_id.is_empty() || access_key.is_empty() {
        return None;
    }
    Some(AuthStrategy::RakutenAppAuth {
        application_id,
        access_key,
    })
}

#[tokio::test]
async fn search_books_by_isbn_returns_result() {
    let Some(auth) = rakuten_auth() else {
        eprintln!("RAKUTEN_APPLICATION_ID / RAKUTEN_ACCESS_KEY not set, skipping test");
        return;
    };

    let client = RakutenClient::new(auth).expect("client init");

    // 9784088725093: ONE PIECE 1
    let resp = client
        .search_books(SearchBooksRequest {
            isbn: Some("9784088725093".into()),
            ..Default::default()
        })
        .await
        .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert!(!resp.model.is_empty());
    assert_eq!(resp.model[0].isbn.as_deref(), Some("9784088725093"));
}
