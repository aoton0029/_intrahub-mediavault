//! `MediaVaultServer`（`ServerHandler` / `tool_router` の骨格）
//!
//! TASK-0009: 共通結果型と rmcp サーバー骨格

use std::sync::Arc;

use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
use rmcp::tool;
use rmcp::tool_handler;
use rmcp::tool_router;

use crate::api::client::ApiClient;
use crate::result::outcome::Outcome;
use crate::services::add_access_link;
use crate::services::collection_overview;
use crate::services::context;
use crate::services::create_item;
use crate::services::health::check_api_health;
use crate::services::import_external_item;
use crate::services::organize;
use crate::services::relate_items;
use crate::services::search_external_catalog;
use crate::services::search_library;
use crate::services::update_consumption;
use crate::tools::add_access_link::{AddAccessLinkParams, AddAccessLinkResult};
use crate::tools::collection_overview::{CollectionOverviewParams, CollectionOverviewResult};
use crate::tools::create_item::{CreateItemParams, CreateItemResult};
use crate::tools::get_item_context::{GetItemContextParams, ItemContextResult};
use crate::tools::health::HealthResult;
use crate::tools::import_external_item::{ImportExternalItemParams, ImportExternalItemResult};
use crate::tools::organize_item::{OrganizeItemParams, OrganizeItemResult};
use crate::tools::relate_items::{RelateItemsParams, RelateItemsResult};
use crate::tools::search_external_catalog::{
    SearchExternalCatalogParams, SearchExternalCatalogResult,
};
use crate::tools::search_library::{SearchLibraryParams, SearchLibraryResult};
use crate::tools::update_consumption::{UpdateConsumptionParams, UpdateConsumptionResult};

/// 🔵 Intent: REQ-001 / REQ-002・architecture.md MCPサーバー層より。
///    ツールは後続タスク（TASK-0011以降）で `#[tool_router]` ブロックへ追加する。
#[derive(Clone)]
pub struct MediaVaultServer {
    api: Arc<ApiClient>,
    #[allow(
        dead_code,
        reason = "tool_handler マクロは Self::tool_router() を都度呼ぶため、フィールドとしては未使用"
    )]
    tool_router: ToolRouter<Self>,
}

impl MediaVaultServer {
    pub fn new(api: Arc<ApiClient>) -> Self {
        Self {
            api,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl MediaVaultServer {
    /// 🔵 Intent: TASK-0010・mcp-tools.md 11. health より。
    ///    api 到達性の確認に限定し、内部/外部APIキーの有効性は確認しない。
    ///    接続失敗はツール全体のエラーへ昇格させず、常に `Outcome::Success` を返す（EDGE-001）。
    #[tool(
        description = "MediaVault-api への到達性を確認する。他のツールが失敗したとき、原因が接続なのかデータなのかを切り分けるために使う。",
        annotations(read_only_hint = true)
    )]
    async fn health(&self) -> rmcp::Json<HealthResult> {
        let api = check_api_health(&self.api).await;
        rmcp::Json(HealthResult {
            outcome: Outcome::Success,
            mcp_version: env!("CARGO_PKG_VERSION").to_string(),
            api,
        })
    }

    /// 🔵 Intent: TASK-0013・mcp-tools.md 1. search_library より。
    ///    所蔵確認の入口ツール。`search_external_catalog` との使い分けを説明文で明示する。
    #[tool(
        description = "MediaVault に既に登録済みの作品を検索する。所蔵確認に使う。未登録の作品を外部から探す場合は search_external_catalog を使うこと。",
        annotations(read_only_hint = true)
    )]
    async fn search_library(
        &self,
        Parameters(params): Parameters<SearchLibraryParams>,
    ) -> rmcp::Json<SearchLibraryResult> {
        rmcp::Json(search_library::search(&self.api, params).await)
    }

    /// 🔵 Intent: TASK-0015・mcp-tools.md 3. search_external_catalog より。
    ///    所蔵確認(`search_library`)とは明確に分離する(REQ-032 / NFR-202)。レスポンスの
    ///    `source` 固定値と `ExternalCandidate` に `item_id` を持たない型設計の両方で担保する。
    #[tool(
        description = "外部の作品データベース(Annict / TMDb / 楽天ブックス / Steam / NDL)を検索する。所蔵確認ではない。ここで見つかった作品は MediaVault にまだ登録されていない。登録するには import_external_item を呼ぶこと。",
        annotations(read_only_hint = true)
    )]
    async fn search_external_catalog(
        &self,
        Parameters(params): Parameters<SearchExternalCatalogParams>,
    ) -> rmcp::Json<SearchExternalCatalogResult> {
        rmcp::Json(search_external_catalog::search(&self.api, params).await)
    }

    /// 🔵 Intent: TASK-0014・mcp-tools.md 2. get_item_context より。
    ///    9本の api 呼び出しを合成し、1回のツール呼び出しで作品コンテキストを返す（NFR-002）。
    #[tool(
        description = "1つの作品について、詳細・タグ・カテゴリ・ファイル・リンク・関連作品・スタッフ・キャストなどをまとめて取得する。作品について詳しく答える前に呼ぶこと。",
        annotations(read_only_hint = true)
    )]
    async fn get_item_context(
        &self,
        Parameters(params): Parameters<GetItemContextParams>,
    ) -> rmcp::Json<ItemContextResult> {
        rmcp::Json(context::get_context(&self.api, params.item_id).await)
    }

    /// 🔵 Intent: TASK-0016・mcp-tools.md 10. collection_overview より。
    ///    候補相談の入口。1回の呼び出しで完結し、推薦や傾向分析は行わない（REQ-144）。
    #[tool(
        description = "コレクション全体の統計と最近の動きを取得する。「何を見るか」を相談される前に呼ぶと、候補を絞る条件を立てやすい。",
        annotations(read_only_hint = true)
    )]
    async fn collection_overview(
        &self,
        Parameters(params): Parameters<CollectionOverviewParams>,
    ) -> rmcp::Json<CollectionOverviewResult> {
        rmcp::Json(collection_overview::get_overview(&self.api, params).await)
    }

    /// 🔵 Intent: TASK-0017・mcp-tools.md 4. import_external_item より。
    ///    **最初の書き込み系ツール**。annotation は書き込み・非破壊
    ///    （既存データを削除・上書きしない）を明示する。409 は冪等化し、
    ///    重複登録は成功として扱う（REQ-112）。
    #[tool(
        description = "search_external_catalog で見つけた候補を MediaVault へ登録する。利用者が候補を1つ選んでから呼ぶこと。",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn import_external_item(
        &self,
        Parameters(params): Parameters<ImportExternalItemParams>,
    ) -> rmcp::Json<ImportExternalItemResult> {
        rmcp::Json(import_external_item::import(&self.api, params).await)
    }

    /// 🔵 Intent: TASK-0018・mcp-tools.md 5. create_item より。
    ///    外部で見つからない作品の手動登録用。**冪等ではない**（同じ引数で2回呼ぶと
    ///    Item が2件作られる）ため、説明文で再実行の危険性と使い分けを明示する。
    ///    ロールバックしない（REQ-141）：関連付けが失敗しても作成済み Item は残る。
    #[tool(
        description = "外部データベースに存在しない作品（同人誌・個人資料・ローカルファイルなど）を手動で登録する。外部で見つかる作品は search_external_catalog → import_external_item を使うこと。このツールは冪等ではない: 同じ引数で2回呼ぶと作品が2件登録される。失敗時に安易に再実行せず、返された item_id を確認すること。",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn create_item(
        &self,
        Parameters(params): Parameters<CreateItemParams>,
    ) -> rmcp::Json<CreateItemResult> {
        rmcp::Json(create_item::create(&self.api, params).await)
    }

    /// 🔵 Intent: TASK-0019・mcp-tools.md 6. update_consumption より。
    ///    対象は `item_id`（UUID）のみでタイトル指定は受け付けない（REQ-142 / 設計決定 D-02）。
    ///    更新前後の値を `changes` として返す（REQ-051 / NFR-203）。
    #[tool(
        description = "視聴・読了・プレイの状況を記録する。item_id は search_library で先に一意に特定しておくこと。日付は YYYY-MM-DD に変換してから渡すこと（「昨日」などの表現は受け付けない）。",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn update_consumption(
        &self,
        Parameters(params): Parameters<UpdateConsumptionParams>,
    ) -> rmcp::Json<UpdateConsumptionResult> {
        rmcp::Json(update_consumption::update(&self.api, params).await)
    }

    /// 🔵 Intent: TASK-0020・mcp-tools.md 7. organize_item より。
    ///    タグ・カテゴリ・マイリストを**名前で**指定して作品に付与する。既に付与済みなら
    ///    重複を作らず `already_applied` を返す（設計決定 D-03・REQ-113）。
    ///    存在しない名前は既定では作成されない（REQ-111）。ロールバックしない（REQ-141）。
    #[tool(
        description = "タグ・カテゴリ・マイリストを名前で指定して作品に付与する。存在しない名前は既定では作成されず「候補なし」として返る。新規作成したい場合のみ create_if_missing: true を指定すること。",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn organize_item(
        &self,
        Parameters(params): Parameters<OrganizeItemParams>,
    ) -> rmcp::Json<OrganizeItemResult> {
        rmcp::Json(organize::organize(&self.api, params).await)
    }

    /// 🔵 Intent: TASK-0021・mcp-tools.md 8. relate_items より。
    ///    **向きの取り違えが最も事故が起きやすい**（REQ-072）。パラメータ説明文に
    ///    起点・終点の意味を種別ごとに埋め込み、結果の `description` で自然文にして返す。
    ///    409 は冪等化する（REQ-113）。誤登録は MVP では取り消せない（REQ-141）。
    #[tool(
        description = "2つの作品を関係づける。両方の item_id を search_library で先に特定しておくこと。関係には向きがあり、item_id が起点、related_item_id が終点になる。種別ごとの起点・終点の意味: adaptation(原作→映像化・翻案作品), sequel(前作→続編), prequel(後の作品→前日譚), spinoff(本編→スピンオフ), dlc(本編→DLC・追加コンテンツ), reference(引用元→引用先)。誤って逆に登録しても MCP からは解除できない。",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn relate_items(
        &self,
        Parameters(params): Parameters<RelateItemsParams>,
    ) -> rmcp::Json<RelateItemsResult> {
        rmcp::Json(relate_items::relate(&self.api, params).await)
    }

    /// 🔵 Intent: TASK-0022・mcp-tools.md 9. add_access_link より。
    ///    api の配信プラットフォームは5種固定で、Jellyfin・Calibre-Web・U-NEXT などは
    ///    表現できない。非対応時は通常リンクへフォールバックし `fallback_from` で通知する
    ///    （REQ-081）。黙ってフォールバックすると利用者が誤解する（タスクファイルより）。
    #[tool(
        description = "作品に URL を追加する。配信サービスは kind: \"streaming\"、予告編は kind: \"trailer\"、公式サイト・Jellyfin・Calibre-Web などは kind: \"link\" を使う。streaming で対応プラットフォーム(netflix/amazon_prime/disney_plus/dmm_tv/apple_tv)以外を指定した場合は通常リンクへ自動的に切り替わり、fallback_from に \"streaming\" が入る。",
        annotations(read_only_hint = false, destructive_hint = false)
    )]
    async fn add_access_link(
        &self,
        Parameters(params): Parameters<AddAccessLinkParams>,
    ) -> rmcp::Json<AddAccessLinkResult> {
        rmcp::Json(add_access_link::add_access_link(&self.api, params).await)
    }
}

#[tool_handler]
impl ServerHandler for MediaVaultServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                "mediavault-mcp",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions("MediaVault のライブラリを検索・参照・更新するためのMCPサーバー")
    }
}
