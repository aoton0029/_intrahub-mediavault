use api_client_lib::clients::steam::requests::{SteamAppDetailsRequest, SteamStoreSearchRequest};
use api_client_lib::clients::steam::SteamClient;
use api_client_lib::AuthStrategy;

#[tokio::test]
async fn store_search_returns_results() {
    let client = SteamClient::new(AuthStrategy::None).expect("client init");

    let resp = client
        .store_search(SteamStoreSearchRequest {
            term: "Portal".into(),
            page: None,
        })
        .await
        .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert!(!resp.model.is_empty());
}

#[tokio::test]
async fn get_app_details_returns_name() {
    let client = SteamClient::new(AuthStrategy::None).expect("client init");

    // 400: Portal
    let resp = client
        .get_app_details(SteamAppDetailsRequest { app_id: 400 })
        .await
        .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert!(resp.model.name.is_some());
}
