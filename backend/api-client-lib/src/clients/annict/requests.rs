/// Annict 作品一覧取得リクエスト（GET /v1/works）。
#[derive(Debug, Clone, Default)]
pub struct ListWorksRequest {
    pub fields: Option<String>,
    pub filter_ids: Option<String>,
    pub filter_season: Option<String>,
    pub filter_title: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort_id: Option<String>,
    pub sort_season: Option<String>,
    pub sort_watchers_count: Option<String>,
}

/// Annict エピソード一覧取得リクエスト（GET /v1/episodes）。
#[derive(Debug, Clone, Default)]
pub struct ListEpisodesRequest {
    pub fields: Option<String>,
    pub filter_ids: Option<String>,
    pub filter_work_id: Option<u64>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort_id: Option<String>,
    pub sort_sort_number: Option<String>,
}

/// Annict シリーズ一覧取得リクエスト（GET /v1/series）。
#[derive(Debug, Clone, Default)]
pub struct ListSeriesRequest {
    pub fields: Option<String>,
    pub filter_ids: Option<String>,
    pub filter_name: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort_id: Option<String>,
}

/// Annict キャラクター一覧取得リクエスト（GET /v1/characters）。
#[derive(Debug, Clone, Default)]
pub struct ListCharactersRequest {
    pub fields: Option<String>,
    pub filter_ids: Option<String>,
    pub filter_name: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort_id: Option<String>,
}

/// Annict 人物一覧取得リクエスト（GET /v1/people）。
#[derive(Debug, Clone, Default)]
pub struct ListPeopleRequest {
    pub fields: Option<String>,
    pub filter_ids: Option<String>,
    pub filter_name: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort_id: Option<String>,
}

/// Annict 団体一覧取得リクエスト（GET /v1/organizations）。
#[derive(Debug, Clone, Default)]
pub struct ListOrganizationsRequest {
    pub fields: Option<String>,
    pub filter_ids: Option<String>,
    pub filter_name: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort_id: Option<String>,
}

/// Annict キャスト一覧取得リクエスト（GET /v1/casts）。
#[derive(Debug, Clone, Default)]
pub struct ListCastsRequest {
    pub fields: Option<String>,
    pub filter_ids: Option<String>,
    pub filter_work_id: Option<u64>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort_id: Option<String>,
    pub sort_sort_number: Option<String>,
}

/// Annict スタッフ一覧取得リクエスト（GET /v1/staffs）。
#[derive(Debug, Clone, Default)]
pub struct ListStaffsRequest {
    pub fields: Option<String>,
    pub filter_ids: Option<String>,
    pub filter_work_id: Option<u64>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort_id: Option<String>,
    pub sort_sort_number: Option<String>,
}

/// Annict クライアントへのリクエスト列挙型。
#[derive(Debug, Clone)]
pub enum AnnictRequest {
    ListWorks(ListWorksRequest),
    ListEpisodes(ListEpisodesRequest),
    ListSeries(ListSeriesRequest),
    ListCharacters(ListCharactersRequest),
    ListPeople(ListPeopleRequest),
    ListOrganizations(ListOrganizationsRequest),
    ListCasts(ListCastsRequest),
    ListStaffs(ListStaffsRequest),
}
