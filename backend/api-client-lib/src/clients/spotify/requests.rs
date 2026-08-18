use super::SpotifyError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchType {
    Album,
    Artist,
    Track,
}

impl SearchType {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Album => "album",
            Self::Artist => "artist",
            Self::Track => "track",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlbumGroup {
    Album,
    Single,
    AppearsOn,
    Compilation,
}

impl AlbumGroup {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Album => "album",
            Self::Single => "single",
            Self::AppearsOn => "appears_on",
            Self::Compilation => "compilation",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchRequest {
    pub query: String,
    pub types: Vec<SearchType>,
    pub market: Option<String>,
    pub limit: Option<u8>,
    pub offset: Option<u16>,
}

impl SearchRequest {
    pub(crate) fn validate(&self) -> Result<(), SpotifyError> {
        if self.query.trim().is_empty() {
            return Err(SpotifyError::invalid("query", "must not be empty"));
        }
        if self.types.is_empty() {
            return Err(SpotifyError::invalid(
                "types",
                "must contain at least one type",
            ));
        }
        if let Some(limit) = self.limit {
            if !(1..=10).contains(&limit) {
                return Err(SpotifyError::invalid("limit", "must be between 1 and 10"));
            }
        }
        if self.offset.unwrap_or(0) > 1_000 {
            return Err(SpotifyError::invalid(
                "offset",
                "must be between 0 and 1000",
            ));
        }
        validate_market(self.market.as_deref())
    }
}

#[derive(Debug, Clone)]
pub struct ArtistAlbumsRequest {
    pub include_groups: Vec<AlbumGroup>,
    pub market: Option<String>,
    pub limit: Option<u8>,
    pub offset: Option<u16>,
}

impl Default for ArtistAlbumsRequest {
    fn default() -> Self {
        Self {
            include_groups: vec![
                AlbumGroup::Album,
                AlbumGroup::Single,
                AlbumGroup::Compilation,
            ],
            market: None,
            limit: Some(10),
            offset: Some(0),
        }
    }
}

impl ArtistAlbumsRequest {
    pub(crate) fn validate(&self) -> Result<(), SpotifyError> {
        if let Some(limit) = self.limit {
            if !(1..=10).contains(&limit) {
                return Err(SpotifyError::invalid("limit", "must be between 1 and 10"));
            }
        }
        if self.offset.unwrap_or(0) > 1_000 {
            return Err(SpotifyError::invalid(
                "offset",
                "must be between 0 and 1000",
            ));
        }
        validate_market(self.market.as_deref())
    }
}

fn validate_market(market: Option<&str>) -> Result<(), SpotifyError> {
    if let Some(market) = market {
        if market.len() != 2 || !market.bytes().all(|b| b.is_ascii_alphabetic()) {
            return Err(SpotifyError::invalid(
                "market",
                "must be an ISO 3166-1 alpha-2 code",
            ));
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub enum SpotifyRequest {
    Search(SearchRequest),
    Album(String),
    AlbumTracks(String),
    Track(String),
    Artist(String),
    ArtistAlbums(String, ArtistAlbumsRequest),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_request_validation_rejects_invalid_values() {
        let valid = SearchRequest {
            query: "Discovery".into(),
            types: vec![SearchType::Album],
            market: Some("JP".into()),
            limit: Some(10),
            offset: Some(1_000),
        };
        assert!(valid.validate().is_ok());

        let invalid = SearchRequest {
            limit: Some(11),
            ..valid
        };
        assert!(matches!(
            invalid.validate(),
            Err(SpotifyError::InvalidRequest { field: "limit", .. })
        ));
    }

    #[test]
    fn artist_album_defaults_exclude_appears_on() {
        let request = ArtistAlbumsRequest::default();
        assert_eq!(
            request.include_groups,
            vec![
                AlbumGroup::Album,
                AlbumGroup::Single,
                AlbumGroup::Compilation
            ]
        );
    }
}
