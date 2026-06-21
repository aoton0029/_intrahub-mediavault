use api_client_lib::clients::tmdb::requests::{MovieDetailsRequest, SearchMovieRequest};
use api_client_lib::clients::tmdb::TmdbClient;
use api_client_lib::AuthStrategy;

#[tokio::test]
async fn search_movie_returns_results() {
    let Ok(api_key) = std::env::var("TMDB_API_KEY") else {
        eprintln!("TMDB_API_KEY not set, skipping test");
        return;
    };
    if api_key.is_empty() {
        eprintln!("TMDB_API_KEY is empty, skipping test");
        return;
    }

    let client = TmdbClient::new(AuthStrategy::ApiKey(api_key)).expect("client init");

    let resp = client
        .search_movie(SearchMovieRequest {
            query: "Inception".into(),
            language: None,
            page: None,
        })
        .await
        .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert!(!resp.model.is_empty());
}

#[tokio::test]
async fn get_movie_details_returns_title() {
    let Ok(api_key) = std::env::var("TMDB_API_KEY") else {
        eprintln!("TMDB_API_KEY not set, skipping test");
        return;
    };
    if api_key.is_empty() {
        eprintln!("TMDB_API_KEY is empty, skipping test");
        return;
    }

    let client = TmdbClient::new(AuthStrategy::ApiKey(api_key)).expect("client init");

    // 27205: Inception
    let resp = client
        .get_movie_details(MovieDetailsRequest {
            movie_id: 27205,
            language: None,
        })
        .await
        .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert!(resp.model.title.is_some());
}
