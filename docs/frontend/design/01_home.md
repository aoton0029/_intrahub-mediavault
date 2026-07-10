# 01. ホーム

対応モック: `docs/frontend/ui/01_home.html`

## 1. 画面概要 / ルート

アプリのランディング画面。総件数・進行中・視聴済み・お気に入り件数のサマリーと、「最近追加した作品」「進行中」の2つのカード一覧を表示する。ルート: `/`（`AppShell` 配下のインデックスルート）。サイドバーの「ホーム」がactive。

## 2. レイアウト構成

```
<AppShell>
  <Content>
    <StatGrid>                          // .stat-grid
      <StatCard label="総件数" value />
      <StatCard label="進行中" value />
      <StatCard label="読了・視聴済み" value />
      <StatCard label="❤ お気に入り" value isFavorite />
    </StatGrid>

    <SectionHeading title="最近追加した作品" seeAllHref="/media" />
    <MediaGrid density="default">
      <MediaCard … />  // GET /items?limit=6 の結果を表示。badge=media_type表示名、rating=評価(小数第1位)
    </MediaGrid>

    <SectionHeading title="進行中" seeAllHref="/media" />
    <MediaGrid density="default">
      <MediaCard … />  // status_label を tag-pill(色=--color-status-progress)で表示
    </MediaGrid>
  </Content>
</AppShell>
```

`StatCard`/`SectionHeading` は `00_common.md` の共通コンポーネント一覧に追加するホーム専用コンポーネント（他画面では未使用のため画面固有扱い）。

## 3. 表示データ / Props型

```ts
interface HomeStats {
  totalCount: number;
  inProgressCount: number;
  doneCount: number;
  favoriteCount: number;
}

interface MediaSummary {
  id: string;
  mediaType: 'anime' | 'movie' | 'drama' | 'manga' | 'novel' | 'game' | 'academic_book' | 'paper';
  title: string;
  volumeLabel?: string;       // 「紙の上の庭園 第4巻」のような巻数付きタイトル表示に対応
  isFavorite: boolean;
  rating?: number;            // 未評価時は非表示（マージ元HTMLの「塩の記憶」のようにmeta自体を出さない）
  statusLabel?: string;       // 進行中セクションのみ使用
}
```

## 4. 画面固有コンポーネント

- `StatGrid` / `StatCard`（`.stat-grid` / `.stat-card` / `.is-favorite`）
- `SectionHeading`（`.section-heading` + `.see-all`）

## 5. インタラクション仕様

- カードクリックで詳細画面へ遷移（`MediaCard` は `<Link>` でラップ、`Symphonia`カードのようにリンク先が無いモック上のプレースホルダは実装時は必ず `item.id` から生成する）
- 「すべて見る」リンクは `02_general_media.md` の一覧画面へ遷移（進行中セクションは `status=in_progress` クエリ付き）
- `00_common.md` §4のインタラクション（テーマ切替等）はサイドバー経由でこの画面にも適用

## 6. API連携

- 統計: `GET /items` の集計（`【要確認】専用の集計APIは未定義。フロントで複数回フェッチして集計する暫定実装か、バックエンドに集計エンドポイント追加が必要』`）
- 最近追加した作品: `GET /items?limit=6`（モックのHTMLコメントに準拠。sortパラメータ未実装のため既定の並び=作成日時降順に暗黙的に依存）
- 進行中: `GET /items?status=in_progress&limit=6`（モックのHTMLコメントに準拠）

参照: [items.md](../../backend/mediavault-api/items.md)

## 7. Tailwindスタイリング上の注意

- `MediaCard` の `.rating`/`.tag-pill(status)` は評価やステータスが無いアイテムでは非表示にする（「塩の記憶」「緋色の境界、青の余白」の表示例を参照）
- `StatCard.is-favorite` の数値色は `--color-favorite`
