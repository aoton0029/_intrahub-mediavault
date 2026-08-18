#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub limit: u32,
    pub offset: u32,
    pub total: u32,
    pub next: Option<String>,
    pub previous: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtistSummary {
    pub id: String,
    pub name: String,
    pub spotify_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub images: Vec<Image>,
    pub genres: Vec<String>,
    pub followers: Option<u64>,
    pub popularity: Option<u8>,
    pub spotify_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlbumType {
    Album,
    Single,
    Compilation,
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatePrecision {
    Year,
    Month,
    Day,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialDate {
    pub value: String,
    pub precision: DatePrecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlbumSummary {
    pub id: String,
    pub name: String,
    pub album_type: AlbumType,
    pub artists: Vec<ArtistSummary>,
    pub images: Vec<Image>,
    pub release_date: PartialDate,
    pub total_tracks: u32,
    pub spotify_url: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExternalIds {
    pub isrc: Option<String>,
    pub ean: Option<String>,
    pub upc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackSummary {
    pub id: String,
    pub name: String,
    pub artists: Vec<ArtistSummary>,
    pub disc_number: u32,
    pub track_number: u32,
    pub duration_ms: u64,
    pub explicit: bool,
    pub is_local: bool,
    pub is_playable: Option<bool>,
    pub spotify_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Album {
    pub summary: AlbumSummary,
    pub tracks: Vec<TrackSummary>,
    pub external_ids: ExternalIds,
    pub label: Option<String>,
    pub popularity: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    pub summary: TrackSummary,
    pub album: AlbumSummary,
    pub external_ids: ExternalIds,
    pub popularity: Option<u8>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchPage {
    pub albums: Option<Page<AlbumSummary>>,
    pub artists: Option<Page<Artist>>,
    pub tracks: Option<Page<Track>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpotifyModel {
    Search(SearchPage),
    Album(Album),
    AlbumTracks(Vec<TrackSummary>),
    Track(Track),
    Artist(Artist),
    ArtistAlbums(Page<AlbumSummary>),
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(bound(deserialize = "T: serde::Deserialize<'de>"))]
pub(super) struct RawPage<T> {
    #[serde(default)]
    pub items: Vec<T>,
    #[serde(default)]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
    #[serde(default)]
    pub total: u32,
    pub next: Option<String>,
    pub previous: Option<String>,
}

impl<T, U> From<RawPage<T>> for Page<U>
where
    U: From<T>,
{
    fn from(value: RawPage<T>) -> Self {
        Self {
            items: value.items.into_iter().map(Into::into).collect(),
            limit: value.limit,
            offset: value.offset,
            total: value.total,
            next: value.next,
            previous: value.previous,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub(super) struct RawExternalUrls {
    pub spotify: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub(super) struct RawExternalIds {
    pub isrc: Option<String>,
    pub ean: Option<String>,
    pub upc: Option<String>,
}

impl From<RawExternalIds> for ExternalIds {
    fn from(value: RawExternalIds) -> Self {
        Self {
            isrc: value.isrc,
            ean: value.ean,
            upc: value.upc,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct RawImage {
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl From<RawImage> for Image {
    fn from(value: RawImage) -> Self {
        Self {
            url: value.url,
            width: value.width,
            height: value.height,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct RawArtistSummary {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub external_urls: RawExternalUrls,
}

impl From<RawArtistSummary> for ArtistSummary {
    fn from(value: RawArtistSummary) -> Self {
        Self {
            id: value.id,
            name: value.name,
            spotify_url: value.external_urls.spotify,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct RawFollowers {
    pub total: Option<u64>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct RawArtist {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub images: Vec<RawImage>,
    #[serde(default)]
    pub genres: Vec<String>,
    pub followers: Option<RawFollowers>,
    pub popularity: Option<u8>,
    #[serde(default)]
    pub external_urls: RawExternalUrls,
}

impl From<RawArtist> for Artist {
    fn from(value: RawArtist) -> Self {
        Self {
            id: value.id,
            name: value.name,
            images: value.images.into_iter().map(Into::into).collect(),
            genres: value.genres,
            followers: value.followers.and_then(|f| f.total),
            popularity: value.popularity,
            spotify_url: value.external_urls.spotify,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct RawAlbumSummary {
    pub id: String,
    pub name: String,
    pub album_type: String,
    #[serde(default)]
    pub artists: Vec<RawArtistSummary>,
    #[serde(default)]
    pub images: Vec<RawImage>,
    #[serde(default)]
    pub release_date: String,
    pub release_date_precision: Option<String>,
    #[serde(default)]
    pub total_tracks: u32,
    #[serde(default)]
    pub external_urls: RawExternalUrls,
}

impl From<RawAlbumSummary> for AlbumSummary {
    fn from(value: RawAlbumSummary) -> Self {
        let precision = match value.release_date_precision.as_deref() {
            Some("day") => DatePrecision::Day,
            Some("month") => DatePrecision::Month,
            _ if value.release_date.matches('-').count() >= 2 => DatePrecision::Day,
            _ if value.release_date.contains('-') => DatePrecision::Month,
            _ => DatePrecision::Year,
        };
        let album_type = match value.album_type.as_str() {
            "album" => AlbumType::Album,
            "single" => AlbumType::Single,
            "compilation" => AlbumType::Compilation,
            other => AlbumType::Other(other.to_owned()),
        };
        Self {
            id: value.id,
            name: value.name,
            album_type,
            artists: value.artists.into_iter().map(Into::into).collect(),
            images: value.images.into_iter().map(Into::into).collect(),
            release_date: PartialDate {
                value: value.release_date,
                precision,
            },
            total_tracks: value.total_tracks,
            spotify_url: value.external_urls.spotify,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct RawTrackSummary {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub artists: Vec<RawArtistSummary>,
    #[serde(default)]
    pub disc_number: u32,
    #[serde(default)]
    pub track_number: u32,
    #[serde(default)]
    pub duration_ms: u64,
    #[serde(default)]
    pub explicit: bool,
    #[serde(default)]
    pub is_local: bool,
    pub is_playable: Option<bool>,
    #[serde(default)]
    pub external_urls: RawExternalUrls,
}

impl From<RawTrackSummary> for TrackSummary {
    fn from(value: RawTrackSummary) -> Self {
        Self {
            id: value.id,
            name: value.name,
            artists: value.artists.into_iter().map(Into::into).collect(),
            disc_number: value.disc_number,
            track_number: value.track_number,
            duration_ms: value.duration_ms,
            explicit: value.explicit,
            is_local: value.is_local,
            is_playable: value.is_playable,
            spotify_url: value.external_urls.spotify,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct RawAlbum {
    #[serde(flatten)]
    pub summary: RawAlbumSummary,
    pub tracks: RawPage<RawTrackSummary>,
    #[serde(default)]
    pub external_ids: RawExternalIds,
    pub label: Option<String>,
    pub popularity: Option<u8>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub(super) struct RawTrack {
    #[serde(flatten)]
    pub summary: RawTrackSummary,
    pub album: RawAlbumSummary,
    #[serde(default)]
    pub external_ids: RawExternalIds,
    pub popularity: Option<u8>,
}

impl From<RawTrack> for Track {
    fn from(value: RawTrack) -> Self {
        Self {
            summary: value.summary.into(),
            album: value.album.into(),
            external_ids: value.external_ids.into(),
            popularity: value.popularity,
        }
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub(super) struct RawSearchPage {
    pub albums: Option<RawPage<RawAlbumSummary>>,
    pub artists: Option<RawPage<RawArtist>>,
    pub tracks: Option<RawPage<RawTrack>>,
}

impl From<RawSearchPage> for SearchPage {
    fn from(value: RawSearchPage) -> Self {
        Self {
            albums: value.albums.map(Into::into),
            artists: value.artists.map(Into::into),
            tracks: value.tracks.map(Into::into),
        }
    }
}

#[derive(serde::Deserialize)]
pub(super) struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
}

#[derive(serde::Deserialize)]
pub(super) struct ErrorEnvelope {
    pub error: Option<ErrorBody>,
    pub error_description: Option<String>,
}

#[derive(serde::Deserialize)]
pub(super) struct ErrorBody {
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn album_fixture_maps_to_domain_model_with_optional_fields() {
        let raw: RawAlbum = serde_json::from_str(include_str!(
            "../../../../../docs/api-samples/spotify/album.json"
        ))
        .expect("album fixture should deserialize");

        let summary: AlbumSummary = raw.summary.into();
        assert_eq!(summary.name, "Discovery");
        assert_eq!(summary.release_date.precision, DatePrecision::Day);
        assert_eq!(
            summary.spotify_url.as_deref(),
            Some("https://open.spotify.com/album/2noRn2Aes5aoNVsU6iWThc")
        );
        assert_eq!(raw.label, None);
        assert_eq!(raw.popularity, None);
        assert_eq!(raw.external_ids.upc.as_deref(), Some("724384960650"));
    }

    #[test]
    fn search_fixture_maps_each_requested_page() {
        let raw: RawSearchPage = serde_json::from_str(include_str!(
            "../../../../../docs/api-samples/spotify/search_album_track.json"
        ))
        .expect("search fixture should deserialize");
        let page: SearchPage = raw.into();

        assert_eq!(page.albums.as_ref().unwrap().items.len(), 1);
        assert_eq!(
            page.tracks.as_ref().unwrap().items[0]
                .external_ids
                .isrc
                .as_deref(),
            Some("GBDUW0000059")
        );
        assert!(page.artists.is_none());
    }

    #[test]
    fn partial_date_precision_is_preserved() {
        for (date, precision, expected) in [
            ("2001", Some("year"), DatePrecision::Year),
            ("2001-03", Some("month"), DatePrecision::Month),
            ("2001-03-12", Some("day"), DatePrecision::Day),
        ] {
            let raw = RawAlbumSummary {
                id: "id".into(),
                name: "name".into(),
                album_type: "album".into(),
                artists: vec![],
                images: vec![],
                release_date: date.into(),
                release_date_precision: precision.map(str::to_owned),
                total_tracks: 1,
                external_urls: RawExternalUrls::default(),
            };
            assert_eq!(AlbumSummary::from(raw).release_date.precision, expected);
        }
    }
}
