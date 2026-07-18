# 20. 映画 詳細

対応モック: `docs/frontend/ui/20_movie_detail.html`

本画面は `00_common.md` §5「詳細画面共通パターン」に準拠する。`DetailLayout`（`DetailRail` + `DetailMain`）の構成・セクション順序は共通パターン通り。ここでは映画固有の内容のみを記載する。

## 1. 画面概要 / ルート

ルート: `/media/:id`（`mediaType: movie`）。パンくずは「メディア / 映画」。タイトルバーに「編集する」ボタンあり（`btn-accent`、遷移先: メディア編集フォーム）。

## 2. レイアウト構成（共通パターンからの差分）

任意セクション有無（`00_common.md` §5マトリクス通り）: 種別固有情報 ✓ / エピソード構成 ✗ / スタッフ ✓ / 配信 ✓。

セクション順序: 概要 → 種別固有情報 → スタッフ → 関連作品 → 配信 → リソース

## 3. 表示データ / Props型

データ型はAPIレスポンス（`items.md`「details（media_type別JSON形状）」）のフィールド名をそのまま使う（snake_case、変換層なし）:

```ts
interface MovieDetail {
  media_type: 'movie';
  detail: {
    runtime_minutes: number;          // 上映時間
    original_language: string;        // 原語
    production_companies: string[];   // 制作会社
    collection?: string;              // コレクション
    genres: string[];                 // ジャンル
    rating: number | null;
    vote_count: number;                // 評価人数
  } | null;
}
```

種別固有情報の `prop-row` 一覧（6項目）: 上映時間 / 原語 / 制作会社 / コレクション / ジャンル / 評価人数。

## 4. 画面固有コンポーネント

なし（すべて `00_common.md` の共通コンポーネントで構成）

## 5. インタラクション仕様

`00_common.md` §4の全パターンを適用（ステータス切替・評価・お気に入り・タグ/カテゴリ追加削除・リソースタブ）。関連作品・配信・リソースの追加/解除/削除ボタンあり。

## 6. API連携

モックHTMLのコメントに基づく（高確度）:

- 詳細取得: `GET /items/{id}`（Item基本情報 + detail + tags + categories + calibre_links）
- ステータス変更: `PATCH /items/{id}/status`
- 評価・お気に入り・その他: `PATCH /items/{id}`
- タグ: `POST /items/{id}/tags/{tag_id}`（付与）, `DELETE /items/{id}/tags/{tag_id}`（削除）。新規タグ名からの作成は `POST /tags { name }` → 返った`id`で付与
- カテゴリ: `POST /items/{id}/categories/{category_id}`（付与）, `DELETE /items/{id}/categories/{category_id}`（削除）。新規カテゴリ名からの作成は `POST /categories { name }` → 返った`id`で付与
- マイリスト所属表示・解除: 取得 `GET /items/{id}/mylists` → `ApiOk<Mylist[]>`（`Mylist = { id, name, created_at }`）、追加 `POST /mylists/{id}/items { item_id }`、解除 `DELETE /mylists/{id}/items/{item_id}`
- 関連作品: 取得 `GET /items/{id}/relations` → `ApiOk<ItemRelation[]>`、作成 `POST /item-relations { item_id, related_item_id, relation_type }`（`relation_type` は `reference` / `dlc` のみ。続編・前日譚等の関係もモック上は「reference」として登録する運用）、解除 `DELETE /item-relations/{id}`（`ItemRelation.id`を使う。`item_id`ではない点に注意）。`ItemRelation = { id, item_id, related_item_id, relation_type, created_at }`
- 配信: `GET/POST /items/{id}/streaming-links`, `DELETE /items/{id}/streaming-links/{link_id}`（`ItemStreamingLink = { id, item_id, platform, url, created_at }`、`platform`: netflix/amazon_prime/disney_plus/dmm_tv/apple_tv）
- リソース: `GET/POST/DELETE /items/{id}/links`, `/items/{id}/files`, `/items/{id}/trailers`。`ItemLink = { id, item_id, url, label, created_at }`, `ItemFile = { id, item_id, path, label, file_type: 'pdf'|'image'|'other', calibre_book_id, created_at }`, `ItemTrailer = { id, item_id, url, label, created_at }`（pdfのみ `PATCH /items/{id}/files/{file_id}/calibre-link` でCalibre連携）
- スタッフ: 取得 `GET /items/{id}/staff` → `ApiOk<ItemStaff[]>`、追加/解除 `POST/DELETE /items/{id}/staff`（スタッフそのものの管理は別画面）。`Staff = { id, external_id, name, image_url, created_at }`, `ItemStaff = { id, item_id, staff_id, role, character_name }`
- ステータス値（`status`）は `not_started` / `in_progress` / `completed` の3値（`data-model.md`の`ItemStatus`定義。モックHTMLの`data-status="done"`は表示上の仮値で、実際の送受信値は`completed`）

参照: [items.md](../../backend/mediavault-api/items.md), [tags.md](../../backend/mediavault-api/tags.md), [categories.md](../../backend/mediavault-api/categories.md), [item-relations.md](../../backend/mediavault-api/item-relations.md), [item-streaming-links.md](../../backend/mediavault-api/item-streaming-links.md), [item-links.md](../../backend/mediavault-api/item-links.md), [item-files.md](../../backend/mediavault-api/item-files.md), [item-trailers.md](../../backend/mediavault-api/item-trailers.md), [staff.md](../../backend/mediavault-api/staff.md), [mylists.md](../../backend/mediavault-api/mylists.md), [data-model.md](../../backend/mediavault-api/data-model.md)

## 7. Tailwindスタイリング上の注意

- ステータスラベルは映画専用の日本語（視聴中/視聴済）。`00_common.md` の `StatusSwitcher` に渡すラベル文言はメディアタイプごとに異なるため、呼び出し側で `labels` propとして渡す設計にする（アニメ「視聴中」、漫画「読書中」、ゲーム「プレイ中」等）
