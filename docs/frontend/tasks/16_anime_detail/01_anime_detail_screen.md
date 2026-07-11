# 01. アニメ詳細画面（AnimeDetailPage）

対応: 設計書 全節（`docs/frontend/design/16_anime_detail.md`）

依存: `docs/frontend/tasks/00_common/04_detail_layout.md`（`DetailLayout`/`DetailRail`/`DetailMain`/`GroupList`/`StaffList`/`StreamingLinks`/`detailSections.ts`）完了後に着手。

## 前提ファイル

- 参照: `docs/frontend/design/16_anime_detail.md`, `docs/frontend/design/20_movie_detail.md`（§6共通詳細画面パターンの記述部分のみ）, `docs/frontend/ui/16_anime_detail.html`, `docs/frontend/ui/_shared.css`, `frontend/src/index.css`, `docs/backend/mediavault-api/items.md`, `docs/backend/mediavault-api/item-groups.md`, `docs/backend/mediavault-api/item-episodes.md`, `docs/backend/mediavault-api/item-relations.md`, `docs/backend/mediavault-api/staff.md`, `docs/backend/mediavault-api/mylists.md`
- 参照（既存実装、直接import対象）: `frontend/src/components/detail/`（`DetailLayout`, `DetailRail`, `DetailSection`, `DetailMain`, `GroupList`, `EpisodeRow`, `StaffList`, `StreamingLinks`, `RailSection`とそれぞれの型）, `frontend/src/components/shared/`（`StatusSwitcher`, `RatingStars`, `FavoriteToggle`, `TagList`, `RelatedWorksList`, `ResourceTabs`, `EmptyState`）, `frontend/src/config/detailSections.ts`（`detailSectionMatrix.anime`）, `frontend/src/components/layout/AppShell.tsx`, `frontend/src/routes.tsx`, `frontend/src/hooks/useMediaListData.ts`（`fetchItemsPage`等のAPI呼び出しパターンの参考）
- 出力: `frontend/src/pages/AnimeDetailPage.tsx`, `frontend/src/hooks/useAnimeDetailData.ts`（新規。`ItemDetail`取得・シーズン/エピソード・スタッフ・関連作品・配信・リソース・タグ/カテゴリ/マイリストのデータ取得・更新ロジックをまとめる）
- 参照範囲はこの節に列挙したファイルとそこから直接importされるファイルに限る。それ以外のファイルを探すための横断的な探索は行わない。

## タスク一覧

- [x] UIに表示するアイテム（ラベル・見出し・ボタン文言・メッセージ等）は日本語を優先して使用する
- [x] アイコンは`react-icons`を積極的に使用する（見出し・ボタン・ステータス表示・空状態等、視覚的な手がかりが有効な箇所には極力アイコンを添える。モックのSVGコメント`<!-- react-icons/fi: FiXxx -->`に対応するアイコンを使う）
- [x] `frontend/src/hooks/useAnimeDetailData.ts` を実装する。`GET /items/{id}` → `ApiOk<ItemDetail>`（`detail`は`AnimeDetail`形状。`source: "manual"`時は`detail`が`null`になりうるため`null`許容で扱う）を取得し、`GET /items/{id}/groups`・`GET /items/{id}/staff`・`GET /items/{id}/relations`・`GET /items/{id}/streaming-links`・`GET /items/{id}/mylists`を並行取得する
- [x] `PATCH /items/{id}/status`（`status`は`not_started`/`in_progress`/`completed`の3値。ラベルは「未着手/視聴中/視聴済」。モックHTMLの`data-status="done"`は仮値であり実装では使わない）で状態変更するハンドラを実装する
- [x] `PATCH /items/{id}`で評価（`rating`、整数のみ）・お気に入り（`is_favorite`）を更新するハンドラを実装する
- [x] タグ（`POST/DELETE /items/{id}/tags`, `/items/{id}/tags/{tag_id}`）・カテゴリ（`POST/DELETE /items/{id}/categories`, `/items/{id}/categories/{category_id}`）の追加/削除ハンドラを実装する（APIパスは`items.md`を参照して確定させる。設計書側に明記が無い場合は`20_movie_detail.md`§6共通パターンの記述に合わせる）
- [x] マイリスト所属表示・解除（`DELETE /mylists/{id}/items/{item_id}`）・「マイリストに追加」導線（遷移先`/mylists`へのリンクのみ。追加フローは本タスク範囲外）を実装する
- [x] シーズン構成: `GET/POST /items/{id}/groups`（`group_type: 'season'`固定で作成）、各シーズンの話数は`GET/POST /groups/{group_id}/episodes`。`ItemGroup`/`ItemEpisode`はAPIレスポンスのsnake_caseフィールドをそのまま使い変換層を設けない。`GroupList`が期待する`{id, label, episodes: {id, number, title}[]}`形状へマッピングする関数を`useAnimeDetailData.ts`内に実装する（`group_name`→`label`、`episode_number`→`number`（2桁ゼロ埋め文字列）、`title`→`title`）
- [x] グループ単位の「話数を追加」ボタン、リスト末尾の「シーズンを追加」ボタンを実装する（モックの`.group-header`内ボタン・`.doc-section`末尾ボタンに対応。`GroupList`/`EpisodeRow`は現状ボタンslotを持たないため、追加ボタンをどう組み込むかは[02_open_questions.md](02_open_questions.md)に記載の上、`DetailMain`の`groups`スロットではなく`DetailSection`を直接使う等の妥当な方法で仮実装してよい）
- [x] `DetailMain`の`groups`セクション見出しは共通コンポーネント側で固定文言「構成」になっている（設計書は「シーズン構成」を指定）。見出し文言の差分は[02_open_questions.md](02_open_questions.md)に記載し、本タスクでは共通コンポーネントを変更せず現状の見出しのまま実装する
- [x] スタッフ: `GET /items/{id}/staff` → `ApiOk<ItemStaff[]>`を`StaffList`が期待する`{id, label, sub}`形状へマッピングする（`label`はスタッフ氏名、`sub`は`role`または`役職(character_name役)`表記。氏名取得には`staff_id`に対応する`Staff`情報が必要なため、`ItemStaff`単独で氏名が取得できない場合は[02_open_questions.md](02_open_questions.md)に記載する）。追加/解除は`POST/DELETE /items/{id}/staff`
- [x] 関連作品: `GET /items/{id}/relations` → `ApiOk<ItemRelation[]>`を`RelatedWorksList`が期待する形状へマッピングする。作成は`POST /item-relations { item_id, related_item_id, relation_type }`（`relation_type`は`reference`/`dlc`のみ）、解除は`DELETE /item-relations/{id}`（`ItemRelation.id`を使う）
- [x] 配信: `GET/POST /items/{id}/streaming-links`, `DELETE /items/{id}/streaming-links/{link_id}`。`platform`（`netflix`/`amazon_prime`/`disney_plus`/`dmm_tv`/`apple_tv`）から`StreamingLinks`の`label`（日本語表示名、例: Netflix→「Netflix」）へのマッピング表を実装する
- [x] リソース（リンク/ファイル/トレーラー）: `POST/DELETE /items/{id}/links`, `/items/{id}/files`, `/items/{id}/trailers`を`ResourceTabs`の`tabs`形状へマッピングする。`ItemFile`の`file_type: 'pdf'`のみ`PATCH /items/{id}/files/{file_id}/calibre-link`でCalibre連携可能である旨は本タスクでは連携ボタンの配置のみ行い、連携フロー自体の実装は範囲外として[02_open_questions.md](02_open_questions.md)に記載する
- [x] `frontend/src/pages/AnimeDetailPage.tsx` を実装する。`useParams`で`id`を取得し`useAnimeDetailData`を呼び出す。`DetailLayout`に`DetailRail`（`facts`に`StatusSwitcher`/`RatingStars`/`FavoriteToggle`/登録日`.meta-item`/外部API ID等`.meta-item.muted`を渡す）と`DetailMain`（`detailSectionMatrix.anime`に従い`propertyList`は渡さず、`groups`/`staffList`/`relatedWorks`/`streaming`/`resourceTabs`を渡す。`overview`は`AnimeDetail`側に概要フィールドが無いため、設計書に概要の出典が明記されていない旨を[02_open_questions.md](02_open_questions.md)に記載し、`Item`の既存フィールド（例: `description`相当）から暫定的に表示する）を組み込む
- [x] パンくずは「一般メディア / アニメ」、編集ボタンは表示しない（モックのタイトルバーaction無しに合わせる。`routes.tsx`の`handle`で`title`/`actions`を制御する既存パターンに合わせて実装する）
- [x] `frontend/src/routes.tsx` に `path: "media/:id"` のルートを追加し、`element={<AnimeDetailPage />}`とする。他`media_type`向け詳細ページが未実装のため、暫定的に`mediaType`判定なしで本タスクではanime専用ルートとして実装してよい旨、および将来的に`media_type`に応じたページ振り分けが必要になる点を[02_open_questions.md](02_open_questions.md)に記載する
- [x] `frontend/src/index.css`: `_shared.css`に対応クラスが無い場合のみ追加してよい。既存クラスの値は変更しない

## テストリスト

- [x] `AnimeDetailPage.test.tsx`: `useAnimeDetailData`が返すデータで`DetailRail`（タイトル・原題・年）と`DetailMain`（概要・シーズン構成・スタッフ・関連作品・配信・リソース）が描画され、`propertyList`セクション（種別固有情報）が描画されないこと
- [x] `AnimeDetailPage.test.tsx`: ステータス変更操作で`PATCH /items/{id}/status`相当のハンドラが`status: 'completed'`等の正しい値で呼ばれること（`done`ではないことを確認）
- [x] `useAnimeDetailData.test.tsx`: `ItemGroup`/`ItemEpisode`のsnake_caseレスポンスが`GroupList`向け形状に正しくマッピングされること
- [x] `useAnimeDetailData.test.tsx`: `detail`が`null`（手動作成）の場合でもエラーにならず概要等が欠損表示になること
- [x] `tests/e2e/anime-detail.spec.ts`: `yarn dev`起動下で`AnimeDetailPage`を実描画し、`docs/frontend/ui/16_anime_detail.html`と主要構造（`.detail-layout`のrail/main構成、`.doc-section`の並び順が概要→シーズン構成→スタッフ→関連作品→配信→リソースであること、編集ボタンが存在しないこと）が一致することをDOM構造アサーション（`getByRole`/`locator`）で確認する

> Codexメモ: 追加系UIは別画面/モーダル未整備のため、今回の範囲では prompt ベースの最小フローで POST 系ハンドラを接続。
> Codexメモ: スタッフ名・関連作品タイトルは API ドキュメントの旧形にも耐えるよう、拡張レスポンスを優先しつつ fallback を入れて実装。
