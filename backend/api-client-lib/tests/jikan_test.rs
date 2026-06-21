use std::time::Duration;

use api_client_lib::clients::jikan::requests::{JikanAnimeDetailsRequest, JikanAnimeSearchRequest};
use api_client_lib::clients::jikan::JikanClient;

#[tokio::test]
async fn search_anime_returns_results() {
    let client = JikanClient::new().expect("client init");

    let resp = client
        .search_anime(JikanAnimeSearchRequest {
            q: Some("Naruto".into()),
            ..Default::default()
        })
        .await
        .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert!(!resp.model.is_empty());
}

#[tokio::test]
async fn get_anime_details_returns_title() {
    // Jikan のレート制限（3req/s）を避けるため少し待機する
    tokio::time::sleep(Duration::from_millis(400)).await;

    let client = JikanClient::new().expect("client init");

    let resp = client
        .get_anime_details(JikanAnimeDetailsRequest { id: 1 })
        .await
        .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert!(resp.model.title.is_some());
}
