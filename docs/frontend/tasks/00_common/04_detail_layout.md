# 04. DetailLayout（詳細画面共通パターン）

対応: 設計書 §6

依存: [03_shared_components.md](03_shared_components.md) の `StatusSwitcher` / `RatingStars` / `FavoriteToggle` / `TagList` / `PropertyList` / `RelatedWorksList` / `ResourceTabs` を利用するため、それらの完了後に着手。

## 前提ファイル

- 参照: `docs/frontend/ui/16_anime_detail.html`, `17_academic_book_detail.html`, `20_movie_detail.html`, `_shared.css` の `.detail-layout` 系クラス
- 出力: `frontend/src/components/detail/` 配下
- 参照範囲はこの節に列挙したファイルとそこから直接importされるファイルに限る。それ以外のファイルを探すための横断的な探索は行わない。

対象8画面（16, 17, 18, 20〜24）が準拠する正準構成を実装する。個別画面固有のフィールド（種別固有情報の項目名等）は各画面タスクで扱い、ここでは型として`items`を受け取る汎用コンポーネントまでを実装する。

## タスク一覧: レール（`DetailRail`、左カラム・sticky）

- [x] `DetailRail` を実装する（sticky、`.doc-cover` + `.doc-title` + `.doc-original`(任意) + `.rail-facts` + `.rail-divider` + `RailSection[]`のスロット構成）
- [x] `.rail-facts` 内に `StatusSwitcher` / `RatingStars` / `FavoriteToggle` / 登録日等`.meta-item` / 外部API ID等`.meta-item.muted`を組み込む
- [x] `RailSection` を実装する（タグ用に`TagList kind="tag"`、カテゴリ用に`TagList kind="category"`、マイリスト用に所属リスト＋解除ボタン＋追加リンクを差し込めるスロット/props構成にする）

## タスク一覧: メイン（`DetailMain`、右カラム）

- [x] `DetailSection`（`icon`, `title`）— `.doc-section` の汎用セクション枠（概要・関連作品など）
- [x] `DetailMain` を実装する。正準順序（概要 → 種別固有情報 → エピソード/巻構成 → スタッフ → 関連作品 → 配信 → リソース）で各セクションを条件付き描画できるよう、画面種別ごとのセクション有無を`props`（例: `sections: { propertyList?, groupList?, staffList?, streaming?, resourceTabs? }`）で制御する
- [x] `GroupList` / `EpisodeRow` を実装する（`.group-block`/`.group-header`/`.episode-row`、シーズン・巻構成）
- [x] `StaffList`（`members`）を実装する（`.prop-list-item`ベース、`FiUsers`）
- [x] `StreamingLinks`（`links`）を実装する（`.prop-list-item`ベース、`FiTv`）
- [x] `DetailLayout`（`rail`, `main`）を実装する（`.detail-layout` grid、`DetailRail`と`DetailMain`を差し込む最上位コンポーネント）
- [x] 画面別セクション有無マトリクス（設計書§6表）をコード上の設定（例: `frontend/src/config/detailSections.ts`）として定義し、各画面タスクから参照できるようにする

## テストリスト

- [x] `DetailLayout`: `rail`/`main`スロットがそれぞれ正しい位置にレンダリングされる
- [x] `DetailMain`: `sections`propsに応じてセクションが正準順序で描画/非描画される（例: `staffList`未指定時にスタッフセクションが描画されない）
- [x] `GroupList`/`EpisodeRow`: グループ内のエピソード行数が`groups`データと一致する
- [x] `RailSection`（マイリスト）: 解除ボタン押下でコールバックが対象マイリストIDで呼ばれる
- [x] 画面別セクション有無マトリクス設定: anime/movie/drama/manga/novel/game/academic_book/paperの8種で設計書§6表と一致する（スナップショットまたは個別assertion）

> Codexメモ: `DetailMain` は optional props の有無でセクション表示を制御し、表示順だけは常に正準順序に固定した。
