pub mod models;
pub mod requests;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::{StatusCode, Url};
use tokio::sync::Mutex;

use crate::error::ApiError;
use crate::rate_limit::RateLimitState;
use crate::response::{ApiResponse, RawData, RequestResult};
use crate::traits::ApiClient;

use self::models::{
    Album, AlbumSummary, Artist, Page, RawAlbum, RawAlbumSummary, RawArtist, RawPage,
    RawSearchPage, RawTrack, RawTrackSummary, SearchPage, SpotifyModel, TokenResponse, Track,
    TrackSummary,
};
use self::requests::{ArtistAlbumsRequest, SearchRequest, SpotifyRequest};

const API_BASE_URL: &str = "https://api.spotify.com/v1";
const TOKEN_URL: &str = "https://accounts.spotify.com/api/token";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const TOKEN_EARLY_REFRESH: Duration = Duration::from_secs(60);
const MAX_PAGE_COUNT: usize = 100;

#[derive(Debug, thiserror::Error)]
pub enum SpotifyError {
    #[error("missing or invalid configuration: {field}")]
    Configuration { field: &'static str },
    #[error("invalid request field {field}: {reason}")]
    InvalidRequest { field: &'static str, reason: String },
    #[error("Spotify authentication failed")]
    Authentication,
    #[error("Spotify request forbidden")]
    Forbidden,
    #[error("Spotify {resource} not found: {id}")]
    NotFound { resource: &'static str, id: String },
    #[error("Spotify rate limit exceeded; retry after {retry_after:?}")]
    RateLimited { retry_after: Option<Duration> },
    #[error("Spotify upstream error: status={status}, message={message:?}")]
    Upstream {
        status: u16,
        message: Option<String>,
    },
    #[error("Spotify transport error (retryable={retryable}): {source}")]
    Transport {
        retryable: bool,
        #[source]
        source: reqwest::Error,
    },
    #[error("Spotify response decode error: {source}")]
    Decode {
        #[source]
        source: serde_json::Error,
    },
}

impl SpotifyError {
    pub(crate) fn invalid(field: &'static str, reason: impl Into<String>) -> Self {
        Self::InvalidRequest {
            field,
            reason: reason.into(),
        }
    }
}

impl From<SpotifyError> for ApiError {
    fn from(error: SpotifyError) -> Self {
        match error {
            SpotifyError::Authentication => ApiError::Auth("Spotify authentication failed".into()),
            SpotifyError::RateLimited { retry_after } => ApiError::RateLimit { retry_after },
            SpotifyError::Decode { source } => ApiError::Parse(source.to_string()),
            SpotifyError::Transport { source, .. } if source.is_timeout() => ApiError::Timeout,
            SpotifyError::Transport { source, .. } => ApiError::Network(source.to_string()),
            SpotifyError::Forbidden => ApiError::Http {
                status: 403,
                body: String::new(),
            },
            SpotifyError::NotFound { resource, id } => ApiError::Http {
                status: 404,
                body: format!("{resource} not found: {id}"),
            },
            SpotifyError::Upstream { status, message } => ApiError::Http {
                status,
                body: message.unwrap_or_default(),
            },
            other => ApiError::Network(other.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpotifyConfig {
    pub client_id: String,
    pub client_secret: String,
    pub market: String,
    pub api_base_url: String,
    pub token_url: String,
}

impl SpotifyConfig {
    pub fn from_env() -> Result<Self, SpotifyError> {
        let client_id =
            std::env::var("SPOTIFY_CLIENT_ID").map_err(|_| SpotifyError::Configuration {
                field: "SPOTIFY_CLIENT_ID",
            })?;
        let client_secret =
            std::env::var("SPOTIFY_CLIENT_SECRET").map_err(|_| SpotifyError::Configuration {
                field: "SPOTIFY_CLIENT_SECRET",
            })?;
        Ok(Self {
            client_id,
            client_secret,
            market: std::env::var("SPOTIFY_MARKET").unwrap_or_else(|_| "JP".into()),
            api_base_url: std::env::var("SPOTIFY_API_BASE_URL")
                .unwrap_or_else(|_| API_BASE_URL.into()),
            token_url: std::env::var("SPOTIFY_TOKEN_URL").unwrap_or_else(|_| TOKEN_URL.into()),
        })
    }
}

#[derive(Clone)]
struct TokenState {
    access_token: String,
    expires_at: Instant,
}

#[derive(Clone)]
struct CacheEntry {
    value: SpotifyModel,
    expires_at: Instant,
}

/// Spotify catalog client using OAuth 2.0 Client Credentials.
pub struct SpotifyClient {
    http: reqwest::Client,
    client_id: String,
    client_secret: String,
    market: String,
    base_url: Url,
    token_url: Url,
    token: Arc<Mutex<Option<TokenState>>>,
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    negative_cache: Arc<Mutex<HashMap<String, Instant>>>,
    rate_limit: Arc<Mutex<RateLimitState>>,
}

impl SpotifyClient {
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
    ) -> Result<Self, SpotifyError> {
        Self::from_config(SpotifyConfig {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            market: "JP".into(),
            api_base_url: API_BASE_URL.into(),
            token_url: TOKEN_URL.into(),
        })
    }

    pub fn from_env() -> Result<Self, SpotifyError> {
        Self::from_config(SpotifyConfig::from_env()?)
    }

    pub fn from_config(config: SpotifyConfig) -> Result<Self, SpotifyError> {
        if config.client_id.trim().is_empty() {
            return Err(SpotifyError::Configuration {
                field: "SPOTIFY_CLIENT_ID",
            });
        }
        if config.client_secret.trim().is_empty() {
            return Err(SpotifyError::Configuration {
                field: "SPOTIFY_CLIENT_SECRET",
            });
        }
        validate_market(&config.market)?;
        let base_url = Url::parse(config.api_base_url.trim_end_matches('/')).map_err(|_| {
            SpotifyError::Configuration {
                field: "SPOTIFY_API_BASE_URL",
            }
        })?;
        let token_url = Url::parse(&config.token_url).map_err(|_| SpotifyError::Configuration {
            field: "SPOTIFY_TOKEN_URL",
        })?;
        let http = reqwest::Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .user_agent(format!("intrahub-mediavault/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| SpotifyError::Configuration {
                field: "http_client",
            })?;
        Ok(Self {
            http,
            client_id: config.client_id,
            client_secret: config.client_secret,
            market: config.market.to_ascii_uppercase(),
            base_url,
            token_url,
            token: Arc::new(Mutex::new(None)),
            cache: Arc::new(Mutex::new(HashMap::new())),
            negative_cache: Arc::new(Mutex::new(HashMap::new())),
            rate_limit: Arc::new(Mutex::new(RateLimitState::new(29, 30))),
        })
    }

    fn url(&self, segments: &[&str]) -> Result<Url, SpotifyError> {
        let mut url = self.base_url.clone();
        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| SpotifyError::Configuration {
                    field: "SPOTIFY_API_BASE_URL",
                })?;
            path.pop_if_empty();
            for segment in segments {
                path.push(segment);
            }
        }
        Ok(url)
    }

    async fn access_token(&self) -> Result<String, SpotifyError> {
        // The lock deliberately spans token acquisition, providing single-flight refresh.
        let mut state = self.token.lock().await;
        if let Some(token) = state.as_ref() {
            if token.expires_at > Instant::now() {
                return Ok(token.access_token.clone());
            }
        }
        let response = self
            .http
            .post(self.token_url.clone())
            .basic_auth(&self.client_id, Some(&self.client_secret))
            .form(&[("grant_type", "client_credentials")])
            .send()
            .await
            .map_err(|source| SpotifyError::Transport {
                retryable: source.is_timeout() || source.is_connect(),
                source,
            })?;
        if !response.status().is_success() {
            tracing::warn!(
                status = response.status().as_u16(),
                "Spotify token request failed"
            );
            return Err(SpotifyError::Authentication);
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|source| SpotifyError::Transport {
                retryable: false,
                source,
            })?;
        let token: TokenResponse =
            serde_json::from_slice(&bytes).map_err(|_| SpotifyError::Authentication)?;
        let lifetime = Duration::from_secs(token.expires_in).saturating_sub(TOKEN_EARLY_REFRESH);
        *state = Some(TokenState {
            access_token: token.access_token.clone(),
            expires_at: Instant::now() + lifetime,
        });
        Ok(token.access_token)
    }

    async fn invalidate_token(&self) {
        *self.token.lock().await = None;
    }

    async fn get_bytes(&self, url: Url) -> Result<Vec<u8>, SpotifyError> {
        let mut auth_retry = false;
        let mut retry_count = 0u8;
        loop {
            loop {
                let mut limit = self.rate_limit.lock().await;
                match limit.check_and_increment() {
                    Ok(()) => break,
                    Err(ApiError::RateLimit {
                        retry_after: Some(delay),
                    }) => {
                        drop(limit);
                        tokio::time::sleep(delay).await;
                    }
                    Err(_) => unreachable!(),
                }
            }
            let token = self.access_token().await?;
            let response = self
                .http
                .get(url.clone())
                .bearer_auth(token)
                .header("Accept", "application/json")
                .send()
                .await;
            let response = match response {
                Ok(response) => response,
                Err(source) => {
                    let retryable = source.is_timeout() || source.is_connect();
                    if retryable && retry_count < 2 {
                        retry_count += 1;
                        tokio::time::sleep(backoff(retry_count)).await;
                        continue;
                    }
                    return Err(SpotifyError::Transport { retryable, source });
                }
            };
            let status = response.status();
            if status == StatusCode::UNAUTHORIZED && !auth_retry {
                auth_retry = true;
                self.invalidate_token().await;
                continue;
            }
            if status == StatusCode::TOO_MANY_REQUESTS {
                let retry_after = response
                    .headers()
                    .get(reqwest::header::RETRY_AFTER)
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .map(Duration::from_secs);
                if retry_count < 3 {
                    retry_count += 1;
                    tokio::time::sleep(retry_after.unwrap_or_else(|| backoff(retry_count))).await;
                    continue;
                }
                return Err(SpotifyError::RateLimited { retry_after });
            }
            if matches!(status.as_u16(), 500 | 502 | 503 | 504) && retry_count < 3 {
                retry_count += 1;
                tokio::time::sleep(backoff(retry_count)).await;
                continue;
            }
            if status == StatusCode::UNAUTHORIZED {
                return Err(SpotifyError::Authentication);
            }
            if status == StatusCode::FORBIDDEN {
                return Err(SpotifyError::Forbidden);
            }
            let bytes = response
                .bytes()
                .await
                .map_err(|source| SpotifyError::Transport {
                    retryable: false,
                    source,
                })?;
            if !status.is_success() {
                let message = parse_error_message(&bytes);
                return Err(SpotifyError::Upstream {
                    status: status.as_u16(),
                    message,
                });
            }
            return Ok(bytes.to_vec());
        }
    }

    async fn decode<T: serde::de::DeserializeOwned>(&self, url: Url) -> Result<T, SpotifyError> {
        let bytes = self.get_bytes(url).await?;
        serde_json::from_slice(&bytes).map_err(|source| SpotifyError::Decode { source })
    }

    async fn cache_get(&self, key: &str) -> Option<SpotifyModel> {
        let mut cache = self.cache.lock().await;
        match cache.get(key) {
            Some(entry) if entry.expires_at > Instant::now() => Some(entry.value.clone()),
            Some(_) => {
                cache.remove(key);
                None
            }
            None => None,
        }
    }

    async fn cache_put(&self, key: String, value: SpotifyModel, ttl: Duration) {
        self.cache.lock().await.insert(
            key,
            CacheEntry {
                value,
                expires_at: Instant::now() + ttl,
            },
        );
    }

    async fn check_negative(
        &self,
        key: &str,
        resource: &'static str,
        id: &str,
    ) -> Result<(), SpotifyError> {
        let mut cache = self.negative_cache.lock().await;
        if cache
            .get(key)
            .is_some_and(|expires| *expires > Instant::now())
        {
            return Err(SpotifyError::NotFound {
                resource,
                id: id.to_owned(),
            });
        }
        cache.remove(key);
        Ok(())
    }

    async fn decode_resource<T: serde::de::DeserializeOwned>(
        &self,
        url: Url,
        key: &str,
        resource: &'static str,
        id: &str,
    ) -> Result<T, SpotifyError> {
        self.check_negative(key, resource, id).await?;
        match self.decode(url).await {
            Err(SpotifyError::Upstream { status: 404, .. }) => {
                self.negative_cache
                    .lock()
                    .await
                    .insert(key.to_owned(), Instant::now() + Duration::from_secs(300));
                Err(SpotifyError::NotFound {
                    resource,
                    id: id.to_owned(),
                })
            }
            result => result,
        }
    }

    async fn all_track_pages(
        &self,
        first: RawPage<RawTrackSummary>,
    ) -> Result<Vec<TrackSummary>, SpotifyError> {
        let mut tracks: Vec<_> = first.items.into_iter().map(Into::into).collect();
        let mut next = first.next;
        let mut pages = 1usize;
        while let Some(next_url) = next {
            if pages >= MAX_PAGE_COUNT {
                return Err(SpotifyError::Upstream {
                    status: 508,
                    message: Some("Spotify pagination exceeded 100 pages".into()),
                });
            }
            let url = Url::parse(&next_url).map_err(|_| SpotifyError::Upstream {
                status: 502,
                message: Some("Spotify returned an invalid pagination URL".into()),
            })?;
            if url.scheme() != self.base_url.scheme()
                || url.host_str() != self.base_url.host_str()
                || url.port_or_known_default() != self.base_url.port_or_known_default()
            {
                return Err(SpotifyError::Upstream {
                    status: 502,
                    message: Some("Spotify returned a cross-origin pagination URL".into()),
                });
            }
            let page: RawPage<RawTrackSummary> = self.decode(url).await?;
            tracks.extend(page.items.into_iter().map(Into::into));
            next = page.next;
            pages += 1;
        }
        Ok(tracks)
    }
}

#[async_trait::async_trait]
pub trait SpotifyCatalogClient: Send + Sync {
    async fn search(&self, request: SearchRequest) -> Result<SearchPage, SpotifyError>;
    async fn album(&self, id: &str) -> Result<Album, SpotifyError>;
    async fn album_tracks(&self, id: &str) -> Result<Vec<TrackSummary>, SpotifyError>;
    async fn track(&self, id: &str) -> Result<Track, SpotifyError>;
    async fn artist(&self, id: &str) -> Result<Artist, SpotifyError>;
    async fn artist_albums(
        &self,
        id: &str,
        request: ArtistAlbumsRequest,
    ) -> Result<Page<AlbumSummary>, SpotifyError>;
}

#[async_trait::async_trait]
impl SpotifyCatalogClient for SpotifyClient {
    async fn search(&self, request: SearchRequest) -> Result<SearchPage, SpotifyError> {
        request.validate()?;
        let query = request.query.trim();
        let mut type_names: Vec<_> = request.types.iter().map(|kind| kind.as_str()).collect();
        type_names.sort_unstable();
        let mut seen = HashSet::new();
        type_names.retain(|kind| seen.insert(*kind));
        let market = request
            .market
            .as_deref()
            .unwrap_or(&self.market)
            .to_ascii_uppercase();
        let limit = request.limit.unwrap_or(10);
        let offset = request.offset.unwrap_or(0);
        let key = format!(
            "search:{query}:{}:{market}:{limit}:{offset}",
            type_names.join(",")
        );
        if let Some(SpotifyModel::Search(value)) = self.cache_get(&key).await {
            return Ok(value);
        }
        let mut url = self.url(&["search"])?;
        url.query_pairs_mut()
            .append_pair("q", query)
            .append_pair("type", &type_names.join(","))
            .append_pair("market", &market)
            .append_pair("limit", &limit.to_string())
            .append_pair("offset", &offset.to_string());
        let value: SearchPage = self.decode::<RawSearchPage>(url).await?.into();
        self.cache_put(
            key,
            SpotifyModel::Search(value.clone()),
            Duration::from_secs(900),
        )
        .await;
        Ok(value)
    }

    async fn album(&self, id: &str) -> Result<Album, SpotifyError> {
        validate_id(id)?;
        let key = format!("album:{id}");
        if let Some(SpotifyModel::Album(value)) = self.cache_get(&key).await {
            return Ok(value);
        }
        let mut url = self.url(&["albums", id])?;
        url.query_pairs_mut().append_pair("market", &self.market);
        let raw: RawAlbum = self.decode_resource(url, &key, "album", id).await?;
        let summary = raw.summary.into();
        let tracks = self.all_track_pages(raw.tracks).await?;
        let value = Album {
            summary,
            tracks,
            external_ids: raw.external_ids.into(),
            label: raw.label,
            popularity: raw.popularity,
        };
        self.cache_put(
            key,
            SpotifyModel::Album(value.clone()),
            Duration::from_secs(86_400),
        )
        .await;
        Ok(value)
    }

    async fn album_tracks(&self, id: &str) -> Result<Vec<TrackSummary>, SpotifyError> {
        validate_id(id)?;
        let key = format!("album_tracks:{id}");
        if let Some(SpotifyModel::AlbumTracks(value)) = self.cache_get(&key).await {
            return Ok(value);
        }
        let mut url = self.url(&["albums", id, "tracks"])?;
        url.query_pairs_mut()
            .append_pair("market", &self.market)
            .append_pair("limit", "50");
        let raw: RawPage<RawTrackSummary> = self.decode_resource(url, &key, "album", id).await?;
        let value = self.all_track_pages(raw).await?;
        self.cache_put(
            key,
            SpotifyModel::AlbumTracks(value.clone()),
            Duration::from_secs(21_600),
        )
        .await;
        Ok(value)
    }

    async fn track(&self, id: &str) -> Result<Track, SpotifyError> {
        validate_id(id)?;
        let key = format!("track:{id}");
        if let Some(SpotifyModel::Track(value)) = self.cache_get(&key).await {
            return Ok(value);
        }
        let mut url = self.url(&["tracks", id])?;
        url.query_pairs_mut().append_pair("market", &self.market);
        let value: Track = self
            .decode_resource::<RawTrack>(url, &key, "track", id)
            .await?
            .into();
        self.cache_put(
            key,
            SpotifyModel::Track(value.clone()),
            Duration::from_secs(86_400),
        )
        .await;
        Ok(value)
    }

    async fn artist(&self, id: &str) -> Result<Artist, SpotifyError> {
        validate_id(id)?;
        let key = format!("artist:{id}");
        if let Some(SpotifyModel::Artist(value)) = self.cache_get(&key).await {
            return Ok(value);
        }
        let value: Artist = self
            .decode_resource::<RawArtist>(self.url(&["artists", id])?, &key, "artist", id)
            .await?
            .into();
        self.cache_put(
            key,
            SpotifyModel::Artist(value.clone()),
            Duration::from_secs(86_400),
        )
        .await;
        Ok(value)
    }

    async fn artist_albums(
        &self,
        id: &str,
        request: ArtistAlbumsRequest,
    ) -> Result<Page<AlbumSummary>, SpotifyError> {
        validate_id(id)?;
        request.validate()?;
        let groups = request
            .include_groups
            .iter()
            .map(|g| g.as_str())
            .collect::<Vec<_>>()
            .join(",");
        let market = request
            .market
            .as_deref()
            .unwrap_or(&self.market)
            .to_ascii_uppercase();
        let limit = request.limit.unwrap_or(10);
        let offset = request.offset.unwrap_or(0);
        let key = format!("artist_albums:{id}:{groups}:{market}:{limit}:{offset}");
        if let Some(SpotifyModel::ArtistAlbums(value)) = self.cache_get(&key).await {
            return Ok(value);
        }
        let mut url = self.url(&["artists", id, "albums"])?;
        {
            let mut query = url.query_pairs_mut();
            if !groups.is_empty() {
                query.append_pair("include_groups", &groups);
            }
            query
                .append_pair("market", &market)
                .append_pair("limit", &limit.to_string())
                .append_pair("offset", &offset.to_string());
        }
        let value: Page<AlbumSummary> = self
            .decode_resource::<RawPage<RawAlbumSummary>>(url, &key, "artist", id)
            .await?
            .into();
        self.cache_put(
            key,
            SpotifyModel::ArtistAlbums(value.clone()),
            Duration::from_secs(21_600),
        )
        .await;
        Ok(value)
    }
}

impl ApiClient for SpotifyClient {
    type Request = SpotifyRequest;
    type Model = SpotifyModel;

    async fn execute(
        &self,
        request: SpotifyRequest,
    ) -> Result<ApiResponse<SpotifyModel>, ApiError> {
        let model = match request {
            SpotifyRequest::Search(request) => SpotifyModel::Search(self.search(request).await?),
            SpotifyRequest::Album(id) => SpotifyModel::Album(self.album(&id).await?),
            SpotifyRequest::AlbumTracks(id) => {
                SpotifyModel::AlbumTracks(self.album_tracks(&id).await?)
            }
            SpotifyRequest::Track(id) => SpotifyModel::Track(self.track(&id).await?),
            SpotifyRequest::Artist(id) => SpotifyModel::Artist(self.artist(&id).await?),
            SpotifyRequest::ArtistAlbums(id, request) => {
                SpotifyModel::ArtistAlbums(self.artist_albums(&id, request).await?)
            }
        };
        Ok(ApiResponse {
            request: RequestResult {
                status: 200,
                url: self.base_url.to_string(),
                latency_ms: 0,
            },
            raw: RawData::Json(String::new()),
            model,
        })
    }
}

fn validate_id(id: &str) -> Result<(), SpotifyError> {
    if id.trim().is_empty() {
        Err(SpotifyError::invalid("id", "must not be empty"))
    } else {
        Ok(())
    }
}

fn validate_market(market: &str) -> Result<(), SpotifyError> {
    if market.len() == 2 && market.bytes().all(|byte| byte.is_ascii_alphabetic()) {
        Ok(())
    } else {
        Err(SpotifyError::Configuration {
            field: "SPOTIFY_MARKET",
        })
    }
}

fn parse_error_message(bytes: &[u8]) -> Option<String> {
    serde_json::from_slice::<models::ErrorEnvelope>(bytes)
        .ok()
        .and_then(|body| {
            body.error
                .and_then(|error| error.message)
                .or(body.error_description)
        })
}

fn backoff(retry: u8) -> Duration {
    let jitter = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.subsec_millis() as u64 % 101);
    Duration::from_millis(250u64.saturating_mul(1 << retry.saturating_sub(1)) + jitter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn oauth_request_and_successful_get_are_cached() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let mut requests = Vec::new();
            for response_body in [
                r#"{"access_token":"test-token","expires_in":3600}"#,
                include_str!("../../../../../docs/api-samples/spotify/track.json"),
            ] {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut buffer = vec![0; 8192];
                let size = stream.read(&mut buffer).await.unwrap();
                requests.push(String::from_utf8_lossy(&buffer[..size]).into_owned());
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    response_body.len(),
                    response_body
                );
                stream.write_all(response.as_bytes()).await.unwrap();
            }
            requests
        });

        let client = SpotifyClient::from_config(SpotifyConfig {
            client_id: "client-id".into(),
            client_secret: "client-secret".into(),
            market: "JP".into(),
            api_base_url: format!("http://{address}/v1"),
            token_url: format!("http://{address}/token"),
        })
        .unwrap();

        let first = client.track("0DiWol3AO6WpXZgp0goxAV").await.unwrap();
        let second = client.track("0DiWol3AO6WpXZgp0goxAV").await.unwrap();
        assert_eq!(first, second);

        let requests = server.await.unwrap();
        assert!(requests[0].starts_with("POST /token HTTP/1.1"));
        assert!(requests[0]
            .to_ascii_lowercase()
            .contains("authorization: basic "));
        assert!(requests[1].starts_with("GET /v1/tracks/0DiWol3AO6WpXZgp0goxAV?market=JP HTTP/1.1"));
        assert!(requests[1]
            .to_ascii_lowercase()
            .contains("authorization: bearer test-token"));
    }
}
