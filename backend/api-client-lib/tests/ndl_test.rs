use api_client_lib::clients::ndl::requests::NdlSearchRequest;
use api_client_lib::clients::ndl::NdlClient;

#[tokio::test]
async fn search_returns_results() {
    let client = NdlClient::new().expect("client init");

    let resp = client
        .search(NdlSearchRequest {
            title: Some("夏目漱石".into()),
            cnt: Some(5),
            ..Default::default()
        })
        .await
        .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert!(!resp.model.is_empty());
}
