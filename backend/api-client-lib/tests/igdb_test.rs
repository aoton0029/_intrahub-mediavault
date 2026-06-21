use api_client_lib::clients::igdb::requests::GamesQueryRequest;
use api_client_lib::clients::igdb::IgdbClient;

#[tokio::test]
async fn query_games_returns_results() {
    let (Ok(client_id), Ok(client_secret)) = (
        std::env::var("IGDB_CLIENT_ID"),
        std::env::var("IGDB_CLIENT_SECRET"),
    ) else {
        eprintln!("IGDB_CLIENT_ID / IGDB_CLIENT_SECRET not set, skipping test");
        return;
    };
    if client_id.is_empty() || client_secret.is_empty() {
        eprintln!("IGDB_CLIENT_ID / IGDB_CLIENT_SECRET is empty, skipping test");
        return;
    }

    let client = IgdbClient::new(client_id, client_secret).expect("client init");

    let resp = client
        .query_games(GamesQueryRequest {
            query: r#"search "Halo"; fields id,name; limit 5;"#.into(),
        })
        .await
        .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert!(!resp.model.is_empty());
}
