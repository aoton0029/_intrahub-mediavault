/// TMDb 映画モデル（フラット構造体）。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MovieModel {
    pub id: u32,
    pub title: Option<String>,
    pub original_title: Option<String>,
    pub overview: Option<String>,
    pub release_date: Option<String>,
    pub vote_average: Option<f64>,
    pub poster_path: Option<String>,
    pub genre_ids: Option<Vec<u32>>,
}

/// TMDb TV シリーズモデル（フラット構造体）。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TvModel {
    pub id: u32,
    pub name: Option<String>,
    pub original_name: Option<String>,
    pub overview: Option<String>,
    pub first_air_date: Option<String>,
    pub vote_average: Option<f64>,
    pub number_of_seasons: Option<u32>,
    pub number_of_episodes: Option<u32>,
    pub poster_path: Option<String>,
}

/// TMDb TV シーズンモデル（フラット構造体）。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TvSeasonModel {
    pub id: u32,
    pub season_number: u32,
    pub name: Option<String>,
    pub overview: Option<String>,
    pub air_date: Option<String>,
    pub episode_count: Option<u32>,
    pub poster_path: Option<String>,
}

/// TMDb クライアントが返すモデル列挙型。
#[derive(Debug)]
pub enum TmdbModel {
    /// `search_movie` の結果
    MovieList(Vec<MovieModel>),
    /// `get_movie_details` の結果
    MovieDetails(MovieModel),
    /// `search_tv` の結果
    TvList(Vec<TvModel>),
    /// `get_tv_series` の結果
    TvSeries(TvModel),
    /// `get_tv_season` の結果
    TvSeason(TvSeasonModel),
}

// ── 内部デシリアライズ用ラッパー ──

#[derive(serde::Deserialize)]
pub(super) struct TmdbPagedResults<T> {
    pub results: Vec<T>,
}
