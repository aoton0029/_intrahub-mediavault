use api_client_lib::clients::openlibrary::requests::{
    OlIsbnRequest, OlSearchRequest, OlWorksRequest,
};
use api_client_lib::clients::openlibrary::OpenLibraryClient;

#[tokio::test]
async fn search_returns_results() {
    let client = OpenLibraryClient::new().expect("client init");

    let resp = client
        .search(OlSearchRequest {
            q: "Rust programming language".into(),
            page: None,
            limit: Some(5),
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
    let resp = client
        .get_by_isbn(OlIsbnRequest {
            isbn: "9780134685991".into(),
        })
        .await
        .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert!(resp.model.title.is_some());
}

#[tokio::test]
async fn get_works_returns_work() {
    let client = OpenLibraryClient::new().expect("client init");

    let resp = client
        .get_works(OlWorksRequest {
            olid: "OL45804W".into(),
        })
        .await
        .expect("request failed");

    assert_eq!(resp.request.status, 200);
    assert!(resp.model.title.is_some());
}
