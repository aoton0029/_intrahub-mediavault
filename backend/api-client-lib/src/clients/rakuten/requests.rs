/// 楽天ブックス書籍検索リクエスト（GET /BooksBook/Search/20170404）。
///
/// `title` / `author` / `publisher_name` / `isbn` / `books_genre_id` / `size` の
/// いずれか1つ以上を指定する必要がある（楽天API側の制約）。
#[derive(Debug, Clone, Default)]
pub struct SearchBooksRequest {
    pub title: Option<String>,
    pub author: Option<String>,
    pub publisher_name: Option<String>,
    pub isbn: Option<String>,
    pub books_genre_id: Option<String>,
    pub size: Option<String>,
    pub sort: Option<String>,
    pub page: Option<u32>,
    pub hits: Option<u32>,
}

/// 楽天クライアントへのリクエスト列挙型。
#[derive(Debug, Clone)]
pub enum RakutenRequest {
    SearchBooks(SearchBooksRequest),
}
