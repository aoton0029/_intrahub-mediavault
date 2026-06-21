/// TMDb 映画検索リクエスト（GET /search/movie）。
#[derive(Debug, Clone)]
pub struct SearchMovieRequest {
    pub query: String,
    pub language: Option<String>,
    pub page: Option<u32>,
}

/// TMDb 映画詳細取得リクエスト（GET /movie/{movie_id}）。
#[derive(Debug, Clone)]
pub struct MovieDetailsRequest {
    pub movie_id: u32,
    pub language: Option<String>,
}

/// TMDb TV シリーズ検索リクエスト（GET /search/tv）。
#[derive(Debug, Clone)]
pub struct SearchTvRequest {
    pub query: String,
    pub language: Option<String>,
    pub page: Option<u32>,
}

/// TMDb TV シリーズ詳細取得リクエスト（GET /tv/{series_id}）。
#[derive(Debug, Clone)]
pub struct TvSeriesRequest {
    pub series_id: u32,
    pub language: Option<String>,
}

/// TMDb TV シーズン詳細取得リクエスト（GET /tv/{series_id}/season/{season_number}）。
#[derive(Debug, Clone)]
pub struct TvSeasonRequest {
    pub series_id: u32,
    pub season_number: u32,
    pub language: Option<String>,
}

/// TMDb クライアントへのリクエスト列挙型。
#[derive(Debug, Clone)]
pub enum TmdbRequest {
    SearchMovie(SearchMovieRequest),
    GetMovieDetails(MovieDetailsRequest),
    SearchTv(SearchTvRequest),
    GetTvSeries(TvSeriesRequest),
    GetTvSeason(TvSeasonRequest),
}
