use api_client_lib::auth::AuthStrategy;
use api_client_lib::clients::anilist::requests::MediaType;
use api_client_lib::clients::anilist::requests::{MediaDetailsRequest, MediaSearchRequest};
use api_client_lib::clients::anilist::AniListClient;

#[tokio::test]
async fn search_media_returns_results() {
    let client = AniListClient::new(AuthStrategy::None).expect("client init");

    let resp = client
        .search_media(MediaSearchRequest {
            search: Some("Naruto".into()),
            media_type: Some(MediaType::Anime),
            ..Default::default()
        })
        .await
        .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert!(!resp.model.is_empty());
}

#[tokio::test]
async fn get_media_details_returns_media() {
    let client = AniListClient::new(AuthStrategy::None).expect("client init");

    let resp = client
        .get_media_details(MediaDetailsRequest { id: 1 })
        .await
        .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert_eq!(resp.model.id, 1);
}
