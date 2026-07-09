/// 楽天ブックス 書籍モデル。
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookModel {
    pub title: Option<String>,
    pub title_kana: Option<String>,
    pub sub_title: Option<String>,
    pub sub_title_kana: Option<String>,
    pub series_name: Option<String>,
    pub series_name_kana: Option<String>,
    pub contents: Option<String>,
    pub author: Option<String>,
    pub author_kana: Option<String>,
    pub publisher_name: Option<String>,
    pub size: Option<String>,
    pub isbn: Option<String>,
    pub item_caption: Option<String>,
    /// 発売日（日本語表記の文字列。例: "1997年12月24日"）
    pub sales_date: Option<String>,
    pub item_price: Option<u32>,
    pub list_price: Option<u32>,
    pub discount_rate: Option<u32>,
    pub discount_price: Option<u32>,
    pub item_url: Option<String>,
    pub affiliate_url: Option<String>,
    pub small_image_url: Option<String>,
    pub medium_image_url: Option<String>,
    pub large_image_url: Option<String>,
    pub chirayomi_url: Option<String>,
    /// 在庫状況コード（文字列で返却される）
    pub availability: Option<String>,
    pub postage_flag: Option<u32>,
    pub limited_flag: Option<u32>,
    pub review_count: Option<u32>,
    /// 平均評価（文字列で返却される。例: "4.67"）
    pub review_average: Option<String>,
    pub books_genre_id: Option<String>,
}

/// 楽天クライアントが返すモデル列挙型。
#[derive(Debug)]
pub enum RakutenModel {
    Books(Vec<BookModel>),
}

// ── 内部デシリアライズ用ラッパー（ページネーション情報を含む） ──

#[derive(serde::Deserialize)]
pub(super) struct ItemWrapper {
    #[serde(rename = "Item")]
    pub item: BookModel,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub(super) struct BooksSearchResponse {
    pub count: Option<u32>,
    pub page: Option<u32>,
    pub first: Option<u32>,
    pub last: Option<u32>,
    pub hits: Option<u32>,
    pub carrier: Option<u32>,
    pub page_count: Option<u32>,
    #[serde(rename = "Items")]
    pub items: Vec<ItemWrapper>,
    /// ジャンル階層情報。構造が非公開のため生JSONのまま保持する。
    #[serde(rename = "GenreInformation", default)]
    pub genre_information: Vec<serde_json::Value>,
}
