//! 外部 API レスポンスをノーマライズしたドメインモデル
//!
//! docs/api-samples/ 配下の実レスポンス（fetch_samples example で取得）を根拠に、
//! 全プロバイダ共通の [`core::MediaCore`] と、メディア種別ごとにそれを拡張した
//! 詳細モデル（AnimeDetails / MangaDetails / MovieDetails / DramaDetails /
//! GameDetails / NovelDetails）を提供する。
//!
//! `GET /items/search` のレスポンス・`POST /items/import` のリクエストは
//! これらを束ねた [`media_details::MediaDetails`]（media_type が判別子）を使用する。

pub mod anime;
pub mod core;
pub mod drama;
pub mod game;
pub mod manga;
pub mod media_details;
pub mod movie;
pub mod novel;

pub use anime::AnimeDetails;
pub use core::MediaCore;
pub use drama::DramaDetails;
pub use game::GameDetails;
pub use manga::MangaDetails;
pub use media_details::MediaDetails;
pub use movie::MovieDetails;
pub use novel::NovelDetails;

#[cfg(test)]
pub(crate) mod test_util {
    use serde_json::Value;
    use std::path::Path;

    /// docs/api-samples/ 配下の fixture を読み込む。
    /// 未取得（fetch_samples がスキップした）ファイルは None を返しテストをスキップさせる。
    pub fn load_fixture(rel_path: &str) -> Option<Value> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/api-samples")
            .join(rel_path);
        let text = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&text).ok()
    }
}
