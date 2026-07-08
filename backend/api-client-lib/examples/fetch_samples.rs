//! 各外部 API を実際に呼び出し、生レスポンスを docs/api-samples/ に保存するツール。
//!
//! 実行: `cargo run -p api-client-lib --example fetch_samples`
//! API キーは backend/.env（TMDB_API_KEY, IGDB_CLIENT_ID, IGDB_CLIENT_SECRET,
//! STEAM_API_KEY, STEAM_USER_ID）から読み込む。未設定のプロバイダはスキップされる。

use std::path::{Path, PathBuf};
use std::time::Duration;

use api_client_lib::auth::AuthStrategy;
use api_client_lib::clients::igdb::requests::{GamesQueryRequest, IgdbSearchRequest};
use api_client_lib::clients::igdb::IgdbClient;
use api_client_lib::clients::jikan::requests::{
    JikanAnimeDetailsRequest, JikanAnimeEpisodesRequest, JikanAnimePicturesRequest,
    JikanAnimeSearchRequest, JikanAnimeVideosRequest, JikanCharacterSearchRequest,
    JikanMangaDetailsRequest, JikanMangaSearchRequest, JikanProducerSearchRequest,
    JikanSeasonRequest,
};
use api_client_lib::clients::jikan::JikanClient;
use api_client_lib::clients::ndl::requests::NdlSearchRequest;
use api_client_lib::clients::ndl::NdlClient;
use api_client_lib::clients::openlibrary::requests::{
    OlIsbnRequest, OlSearchRequest, OlWorksRequest,
};
use api_client_lib::clients::openlibrary::OpenLibraryClient;
use api_client_lib::clients::steam::requests::{
    GetOwnedGamesRequest, SteamAppDetailsRequest, SteamStoreSearchRequest,
};
use api_client_lib::clients::steam::SteamClient;
use api_client_lib::clients::tmdb::requests::{
    MovieDetailsRequest, SearchMovieRequest, SearchTvRequest, TvSeasonRequest, TvSeriesRequest,
};
use api_client_lib::clients::tmdb::TmdbClient;
use api_client_lib::response::RawData;

struct Saver {
    out_dir: PathBuf,
    saved: u32,
    skipped: u32,
}

impl Saver {
    fn new() -> Self {
        let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/api-samples");
        Self {
            out_dir,
            saved: 0,
            skipped: 0,
        }
    }

    fn save(&mut self, provider: &str, endpoint: &str, raw: &RawData) {
        let dir = self.out_dir.join(provider);
        if let Err(e) = std::fs::create_dir_all(&dir) {
            eprintln!("[skip] {provider}/{endpoint}: create_dir failed: {e}");
            self.skipped += 1;
            return;
        }
        let (path, body) = match raw {
            RawData::Json(s) => {
                let pretty = serde_json::from_str::<serde_json::Value>(s)
                    .and_then(|v| serde_json::to_string_pretty(&v))
                    .unwrap_or_else(|_| s.clone());
                (dir.join(format!("{endpoint}.json")), pretty)
            }
            RawData::Xml(s) => (dir.join(format!("{endpoint}.xml")), s.clone()),
        };
        match std::fs::write(&path, body) {
            Ok(()) => {
                println!("[saved] {}", path.display());
                self.saved += 1;
            }
            Err(e) => {
                eprintln!("[skip] {provider}/{endpoint}: write failed: {e}");
                self.skipped += 1;
            }
        }
    }

    fn skip(&mut self, provider: &str, endpoint: &str, reason: impl std::fmt::Display) {
        eprintln!("[skip] {provider}/{endpoint}: {reason}");
        self.skipped += 1;
    }
}

/// レスポンス Result を保存またはスキップとして記録する。
/// 一時的な障害（504 など）に備えて最大 3 回試行する。
macro_rules! record {
    ($saver:expr, $provider:expr, $endpoint:expr, $call:expr) => {{
        let mut attempt = 0;
        loop {
            attempt += 1;
            match $call {
                Ok(resp) => {
                    $saver.save($provider, $endpoint, &resp.raw);
                    break;
                }
                Err(e) if attempt < 3 => {
                    eprintln!("[retry {attempt}] {}/{}: {e}", $provider, $endpoint);
                    tokio::time::sleep(Duration::from_millis(2000)).await;
                }
                Err(e) => {
                    $saver.skip($provider, $endpoint, e);
                    break;
                }
            }
        }
    }};
}

fn env_opt(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

async fn fetch_jikan(s: &mut Saver) {
    let client = match JikanClient::new() {
        Ok(c) => c,
        Err(e) => return s.skip("jikan", "*", e),
    };
    let pause = || tokio::time::sleep(Duration::from_millis(400));

    record!(
        s,
        "jikan",
        "search_anime",
        client
            .search_anime(JikanAnimeSearchRequest {
                q: Some("Fullmetal Alchemist".into()),
                limit: Some(5),
                ..Default::default()
            })
            .await
    );
    pause().await;
    record!(
        s,
        "jikan",
        "anime_details",
        client
            .get_anime_details(JikanAnimeDetailsRequest { id: 9253 })
            .await
    );
    pause().await;
    record!(
        s,
        "jikan",
        "anime_episodes",
        client
            .get_anime_episodes(JikanAnimeEpisodesRequest {
                id: 9253,
                page: None
            })
            .await
    );
    pause().await;
    // /anime/{id}/staff は API 側で利用不可（常に 504）のためスキップ
    s.skip("jikan", "anime_staff", "endpoint unavailable on Jikan side");
    record!(
        s,
        "jikan",
        "anime_pictures",
        client
            .get_anime_pictures(JikanAnimePicturesRequest { id: 5114 })
            .await
    );
    pause().await;
    record!(
        s,
        "jikan",
        "anime_videos",
        client
            .get_anime_videos(JikanAnimeVideosRequest { id: 5114 })
            .await
    );
    pause().await;
    record!(
        s,
        "jikan",
        "search_manga",
        client
            .search_manga(JikanMangaSearchRequest {
                q: Some("Fullmetal Alchemist".into()),
                limit: Some(5),
                ..Default::default()
            })
            .await
    );
    pause().await;
    record!(
        s,
        "jikan",
        "manga_details",
        client
            .get_manga_details(JikanMangaDetailsRequest { id: 25 })
            .await
    );
    pause().await;
    record!(
        s,
        "jikan",
        "search_characters",
        client
            .search_characters(JikanCharacterSearchRequest {
                q: Some("Okabe".into()),
                page: None,
            })
            .await
    );
    pause().await;
    record!(
        s,
        "jikan",
        "search_producers",
        client
            .search_producers(JikanProducerSearchRequest {
                q: Some("Kyoto".into()),
                page: None,
            })
            .await
    );
    pause().await;
    record!(
        s,
        "jikan",
        "season_anime",
        client
            .get_season_anime(JikanSeasonRequest {
                year: 2024,
                season: "spring".into(),
                page: None,
            })
            .await
    );
}

async fn fetch_tmdb(s: &mut Saver) {
    let Some(api_key) = env_opt("TMDB_API_KEY") else {
        return s.skip("tmdb", "*", "TMDB_API_KEY not set");
    };
    let client = match TmdbClient::new(AuthStrategy::ApiKey(api_key)) {
        Ok(c) => c,
        Err(e) => return s.skip("tmdb", "*", e),
    };
    record!(
        s,
        "tmdb",
        "search_movie",
        client
            .search_movie(SearchMovieRequest {
                query: "The Matrix".into(),
                language: Some("ja-JP".into()),
                page: None,
            })
            .await
    );
    record!(
        s,
        "tmdb",
        "movie_details",
        client
            .get_movie_details(MovieDetailsRequest {
                movie_id: 603,
                language: Some("ja-JP".into()),
            })
            .await
    );
    record!(
        s,
        "tmdb",
        "search_tv",
        client
            .search_tv(SearchTvRequest {
                query: "Breaking Bad".into(),
                language: Some("ja-JP".into()),
                page: None,
            })
            .await
    );
    record!(
        s,
        "tmdb",
        "tv_series",
        client
            .get_tv_series(TvSeriesRequest {
                series_id: 1396,
                language: Some("ja-JP".into()),
            })
            .await
    );
    record!(
        s,
        "tmdb",
        "tv_season",
        client
            .get_tv_season(TvSeasonRequest {
                series_id: 1396,
                season_number: 1,
                language: Some("ja-JP".into()),
            })
            .await
    );
}

async fn fetch_openlibrary(s: &mut Saver) {
    let client = match OpenLibraryClient::new() {
        Ok(c) => c,
        Err(e) => return s.skip("openlibrary", "*", e),
    };
    record!(
        s,
        "openlibrary",
        "search",
        client
            .search(OlSearchRequest {
                q: "The Lord of the Rings".into(),
                page: None,
                limit: Some(5),
            })
            .await
    );
    record!(
        s,
        "openlibrary",
        "isbn",
        client
            .get_by_isbn(OlIsbnRequest {
                isbn: "9780261103252".into(),
            })
            .await
    );
    record!(
        s,
        "openlibrary",
        "works",
        client
            .get_works(OlWorksRequest {
                olid: "OL27448W".into(),
            })
            .await
    );
}

async fn fetch_ndl(s: &mut Saver) {
    let client = match NdlClient::new() {
        Ok(c) => c,
        Err(e) => return s.skip("ndl", "*", e),
    };
    record!(
        s,
        "ndl",
        "search_title",
        client
            .search(NdlSearchRequest {
                title: Some("吾輩は猫である".into()),
                cnt: Some(3),
                ..Default::default()
            })
            .await
    );
    record!(
        s,
        "ndl",
        "search_isbn",
        client
            .search(NdlSearchRequest {
                isbn: Some("9784101010014".into()),
                ..Default::default()
            })
            .await
    );
}

async fn fetch_igdb(s: &mut Saver) {
    let (Some(client_id), Some(client_secret)) =
        (env_opt("IGDB_CLIENT_ID"), env_opt("IGDB_CLIENT_SECRET"))
    else {
        return s.skip(
            "igdb",
            "*",
            "IGDB_CLIENT_ID / IGDB_CLIENT_SECRET not set (both required)",
        );
    };
    let client = match IgdbClient::new(client_id, client_secret) {
        Ok(c) => c,
        Err(e) => return s.skip("igdb", "*", e),
    };
    record!(
        s,
        "igdb",
        "query_games",
        client
            .query_games(GamesQueryRequest {
                query: "fields name,summary,storyline,first_release_date,cover.url,genres.name,platforms.name,involved_companies.company.name,involved_companies.developer,involved_companies.publisher,rating,screenshots.url,url,alternative_names.name; search \"Zelda\"; limit 5;".into(),
            })
            .await
    );
    record!(
        s,
        "igdb",
        "search",
        client
            .search(IgdbSearchRequest {
                query: "search \"Zelda\"; fields name,game; limit 5;".into(),
            })
            .await
    );
}

async fn fetch_steam(s: &mut Saver) {
    let auth = match env_opt("STEAM_API_KEY") {
        Some(k) => AuthStrategy::ApiKey(k),
        None => AuthStrategy::None,
    };
    let client = match SteamClient::new(auth) {
        Ok(c) => c,
        Err(e) => return s.skip("steam", "*", e),
    };
    record!(
        s,
        "steam",
        "store_search",
        client
            .store_search(SteamStoreSearchRequest {
                term: "Portal".into(),
                page: None,
            })
            .await
    );
    record!(
        s,
        "steam",
        "app_details",
        client
            .get_app_details(SteamAppDetailsRequest { app_id: 400 })
            .await
    );
    match (env_opt("STEAM_API_KEY"), env_opt("STEAM_USER_ID")) {
        (Some(_), Some(user_id)) => match user_id.parse::<u64>() {
            Ok(steam_id) => {
                record!(
                    s,
                    "steam",
                    "owned_games",
                    client
                        .get_owned_games(GetOwnedGamesRequest {
                            steam_id,
                            include_appinfo: true,
                            include_played_free_games: true,
                        })
                        .await
                );
            }
            Err(e) => s.skip(
                "steam",
                "owned_games",
                format!("invalid STEAM_USER_ID: {e}"),
            ),
        },
        _ => s.skip(
            "steam",
            "owned_games",
            "STEAM_API_KEY / STEAM_USER_ID not set (both required)",
        ),
    }
}

#[tokio::main]
async fn main() {
    // backend/.env（ワークスペース直下）を優先して読み込む
    let ws_env = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.env");
    if dotenvy::from_path(&ws_env).is_err() {
        dotenvy::dotenv().ok();
    }

    let mut saver = Saver::new();
    fetch_jikan(&mut saver).await;
    fetch_tmdb(&mut saver).await;
    fetch_openlibrary(&mut saver).await;
    fetch_ndl(&mut saver).await;
    fetch_igdb(&mut saver).await;
    fetch_steam(&mut saver).await;

    println!(
        "\ndone: {} saved, {} skipped -> {}",
        saver.saved,
        saver.skipped,
        saver.out_dir.display()
    );
}
