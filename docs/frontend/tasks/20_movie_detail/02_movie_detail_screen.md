# 02. 映画詳細画面（MovieDetailPage）

対応: 設計書 §2〜7（`docs/frontend/design/20_movie_detail.md`）

依存: [01_detail_route_dispatch.md](01_detail_route_dispatch.md)（`MediaDetailPage`ディスパッチャ）完了後に着手。`docs/frontend/tasks/00_common/04_detail_layout.md`（`DetailLayout`/`DetailRail`/`DetailMain`/`PropertyList`/`StaffList`/`StreamingLinks`/`detailSections.ts`）が前提。

## 前提ファイル

- 参照: `docs/frontend/design/20_movie_detail.md`, `docs/frontend/design/00_common.md`（§5「詳細画面共通パターン」・§4インタラクション仕様部分のみ）, `docs/frontend/ui/20_movie_detail.html`, `docs/frontend/ui/_shared.css`, `frontend/src/index.css`, `docs/backend/mediavault-api/items.md`, `docs/backend/mediavault-api/tags.md`, `docs/backend/mediavault-api/categories.md`, `docs/backend/mediavault-api/item-relations.md`, `docs/backend/mediavault-api/item-streaming-links.md`, `docs/backend/mediavault-api/item-links.md`, `docs/backend/mediavault-api/item-files.md`, `docs/backend/mediavault-api/item-trailers.md`, `docs/backend/mediavault-api/staff.md`, `docs/backend/mediavault-api/mylists.md`, `docs/backend/mediavault-api/data-model.md`
- 参照（既存実装、直接import対象。実装パターンの参考として全体を読んでよい）: `frontend/src/pages/AnimeDetailPage.tsx`, `frontend/src/hooks/useAnimeDetailData.ts`
- 参照（既存実装、直接import対象）: `frontend/src/components/detail/`（`DetailLayout`, `DetailRail`, `DetailSection`, `DetailMain`, `PropertyList`, `StaffList`, `StreamingLinks`, `RailSection`とそれぞれの型）, `frontend/src/components/shared/`（`StatusSwitcher`, `RatingStars`, `FavoriteToggle`, `TagList`, `RelatedWorksList`, `ResourceTabs`, `EmptyState`）, `frontend/src/config/detailSections.ts`（`detailSectionMatrix.movie`）, `frontend/src/pages/MediaDetailPage.tsx`（[01_detail_route_dispatch.md](01_detail_route_dispatch.md)の出力）
- 出力: `frontend/src/pages/MovieDetailPage.tsx`, `frontend/src/hooks/useMovieDetailData.ts`（新規。`ItemDetail`取得・スタッフ・関連作品・配信・リソース・タグ/カテゴリ/マイリストのデータ取得・更新ロジックをまとめる）
- 参照範囲はこの節に列挙したファイルとそこから直接importされるファイルに限る。それ以外のファイルを探すための横断的な探索は行わない。

## タスク一覧

- [x] UIに表示するアイテム（ラベル・見出し・ボタン文言・メッセージ等）は日本語を優先して使用する
- [x] アイコンは`react-icons`を積極的に使用する（見出し・ボタン・ステータス表示・空状態等、視覚的な手がかりが有効な箇所には極力アイコンを添える。モックのSVGコメント`<!-- react-icons/fi: FiXxx -->`に対応するアイコンを使う）
- [x] `frontend/src/hooks/useMovieDetailData.ts` を実装する。`GET /items/{id}` → `ApiOk<ItemDetail>`（`detail`は下記`MovieDetail`形状。`source: "manual"`時は`detail`が`null`になりうるため`null`許容で扱う）を取得し、`GET /items/{id}/staff`・`GET /items/{id}/relations`・`GET /items/{id}/streaming-links`・`GET /items/{id}/mylists`を並行取得する

  ```ts
  interface MovieDetail {
    media_type: 'movie';
    detail: {
      runtime_minutes: number;
      original_language: string;
      production_companies: string[];
      collection?: string;
      genres: string[];
      rating: number | null;
      vote_count: number;
    } | null;
  }
  ```

- [x] `PATCH /items/{id}/status`（`status`は`not_started`/`in_progress`/`completed`の3値。ラベルは映画専用の日本語「未着手/視聴中/視聴済」。モックHTMLの`data-status="done"`は仮値であり実装では使わない。`labels`propとして`StatusSwitcher`に渡す）で状態変更するハンドラを実装する
- [x] `PATCH /items/{id}`で評価（`rating`、整数のみ）・お気に入り（`is_favorite`）を更新するハンドラを実装する
- [x] タグ（`POST/DELETE /items/{id}/tags/{tag_id}`）・カテゴリ（`POST/DELETE /items/{id}/categories/{category_id}`）の追加/削除ハンドラを実装する。新規タグ/カテゴリ名からの作成は`POST /tags { name }` / `POST /categories { name }` → 返った`id`で付与する
- [x] マイリスト所属表示・解除（`GET /items/{id}/mylists` → `ApiOk<Mylist[]>`、`DELETE /mylists/{id}/items/{item_id}`）・「マイリストに追加」導線（遷移先`/mylists`へのリンクのみ。追加フローは本タスク範囲外）を実装する
- [x] 種別固有情報（`PropertyList`、6項目）: 上映時間（`runtime_minutes`、「◯分」表記）／原語（`original_language`）／制作会社（`production_companies`、複数はカンマ区切り等で表示）／コレクション（`collection`、無ければ「未登録」等の表示）／ジャンル（`genres`、複数表示）／評価人数（`vote_count`、「◯人」表記）を`PropertyList`が期待する`items`形状にマッピングする。`detail`が`null`の場合は空表示にする
- [x] スタッフ: `GET /items/{id}/staff` → `ApiOk<ItemStaff[]>`（`useAnimeDetailData.ts`と同じレスポンス拡張前提で氏名を含む）を`StaffList`が期待する`{id, label, sub}`形状へマッピングする。追加/解除は`POST/DELETE /items/{id}/staff`
- [x] 関連作品: `GET /items/{id}/relations` → `ApiOk<ItemRelation[]>`を`RelatedWorksList`が期待する形状へマッピングする。作成は`POST /item-relations { item_id, related_item_id, relation_type }`（`relation_type`は`reference`/`dlc`のみ。続編・前日譚等の関係もモック上は「reference」として登録する）、解除は`DELETE /item-relations/{id}`（`ItemRelation.id`を使う。`item_id`ではない点に注意）
- [x] 配信: `GET/POST /items/{id}/streaming-links`, `DELETE /items/{id}/streaming-links/{link_id}`。`platform`（`netflix`/`amazon_prime`/`disney_plus`/`dmm_tv`/`apple_tv`）から`StreamingLinks`の`label`（日本語表示名）へのマッピング表を実装する（`useAnimeDetailData.ts`のマッピング表を流用してよい）
- [x] リソース（リンク/ファイル/トレーラー）: `GET/POST/DELETE /items/{id}/links`, `/items/{id}/files`, `/items/{id}/trailers`を`ResourceTabs`の`tabs`形状へマッピングする。`ItemFile`の`file_type: 'pdf'`のみ`PATCH /items/{id}/files/{file_id}/calibre-link`でCalibre連携可能だが、本タスクではボタン配置のみとし連携フロー自体は範囲外とする（`useAnimeDetailData.ts`の実装方針を踏襲）
- [x] `frontend/src/pages/MovieDetailPage.tsx` を実装する。`useParams`で`id`を取得し`useMovieDetailData`を呼び出す。`DetailLayout`に`DetailRail`（`facts`に`StatusSwitcher`/`RatingStars`/`FavoriteToggle`/登録日`.meta-item`/外部API ID等`.meta-item.muted`を渡す）と`DetailMain`（`detailSectionMatrix.movie`に従いセクション順序「概要 → 種別固有情報 → スタッフ → 関連作品 → 配信 → リソース」で`propertyList`/`staffList`/`relatedWorks`/`streaming`/`resourceTabs`を渡す。`groups`は渡さない）を組み込む。概要（`overview`）は`Item.description`を出典とする
- [x] パンくずは「メディア / 映画」（[01_detail_route_dispatch.md](01_detail_route_dispatch.md)側のディスパッチ実装に従う）、タイトルバーに「編集する」ボタン（`btn-accent`、遷移先: `/media/:id/edit`）を表示する（表示制御自体は[01_detail_route_dispatch.md](01_detail_route_dispatch.md)側で実装するため、本タスクでは`MovieDetailPage`が必要なpropsやハンドルを提供する形で連携する）
- [x] `frontend/src/index.css`: `_shared.css`に対応クラスが無い場合のみ追加してよい。既存クラスの値は変更しない

## テストリスト

- [x] `MovieDetailPage.test.tsx`: `useMovieDetailData`が返すデータで`DetailRail`（タイトル・原題・年）と`DetailMain`（概要・種別固有情報・スタッフ・関連作品・配信・リソース、セクション順序含む）が描画され、`groups`セクション（シーズン構成相当）が描画されないこと
- [x] `MovieDetailPage.test.tsx`: ステータス変更操作で`PATCH /items/{id}/status`相当のハンドラが`status: 'completed'`等の正しい値で呼ばれ、ラベルが「視聴中/視聴済」等の映画専用文言であること（`done`ではないことを確認）
- [x] `useMovieDetailData.test.tsx`: `MovieDetail.detail`のsnake_caseレスポンス（`runtime_minutes`/`original_language`/`production_companies`/`collection`/`genres`/`vote_count`）が`PropertyList`向け6項目に正しくマッピングされること
- [x] `useMovieDetailData.test.tsx`: `detail`が`null`（手動作成）の場合でもエラーにならず種別固有情報が欠損表示になること
- [x] `tests/e2e/movie-detail.spec.ts`: `yarn dev`起動下で`MovieDetailPage`を実描画し、`docs/frontend/ui/20_movie_detail.html`と主要構造（`.detail-layout`のrail/main構成、`.doc-section`の並び順が概要→種別固有情報→スタッフ→関連作品→配信→リソースであること、編集ボタンが存在すること）が一致することをDOM構造アサーション（`getByRole`/`locator`）で確認する

> Codexメモ: タグ/カテゴリ追加は設計書とAPIドキュメントに合わせて `POST /tags` / `POST /categories` で作成後、`POST /items/{id}/tags/{tag_id}` / `POST /items/{id}/categories/{category_id}` で付与する二段階フローにした。既存アニメ詳細の簡略化実装とは揃えていない。
