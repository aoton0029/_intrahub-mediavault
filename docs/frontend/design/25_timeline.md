# 25. 年表（タイムライン）

対応モック: 未作成（本設計書が先行。実装時に `docs/frontend/ui/25_timeline.html` を起こす場合は本書 §2 / §7 を基準とする）

## 1. 画面概要 / ルート

メディア種別（`media_type`）ごとのレーンを縦に並べ、横軸に年を取ったスイムレーン型の年表画面。所蔵作品を「いつの作品か」で俯瞰し、年ごとの偏りやメディアをまたいだ同年比較をひと目でスキャンできるようにする。

- ルート: `/collection/timeline`
- サイドバー: 「コレクション > 年表」がactive
  - 実装時に `frontend/src/config/navigation.tsx` の `コレクション` セクションへ `{ label: "年表", to: "/collection/timeline", icon: FiBarChart2 }` を「年別」の下に追加する
- 既存の「年別」（`/collection/yearly`）との棲み分け:
  - **年別** = 年を1つ選んでその年の作品をグリッドで見る（深掘り）
  - **年表** = 全年 × 全メディアを一望する（俯瞰）。年ラベルクリックで年別画面へ送り込む

【要確認】タイトルバーの表記は「年表」で確定。サイドバーのアイコンは `react-icons/fi` に横棒グラフ系の適当な候補が `FiBarChart2` しか無いため、モック作成時に再検討する。

## 2. レイアウト構成

```
<AppShell>
  <Titlebar title="年表" />
  <Content>
    <FilterToolbar>                          // 共通 FilterToolbar（00_common.md §3）
      <FilterBar>
        <FilterSelect label="基準" options={[リリース年, 視聴・読了年, どちらか]} />  // date_field
        <Chip>❤ お気に入り</Chip>
        <FilterSelect label="状態" options={[未着手, 進行中, 完了]} />
        <Chip active removable>🏷️ {tagName} ×</Chip>
        <ChipAdd>+ タグ</ChipAdd>
        <ChipAdd>+ カテゴリ</ChipAdd>
      </FilterBar>
      <SortSearchGroup>
        <YearRangeSelect />                  // 表示年範囲（既定: 直近20年 / 全期間 / 10年単位）
        <LaneVisibilityDropdown />           // レーンに出す media_type の複数選択
        <DensityToggle />                    // 年カラム幅 compact / default / wide
      </SortSearchGroup>
    </FilterToolbar>

    <TimelineBoard>                          // 横スクロールコンテナ（overflow-x: auto）
      <TimelineAxis>                         // sticky top（年軸）
        <TimelineAxisCorner />               // 左上の固定コーナー（レーンヘッダ列の見出し）
        <TimelineAxisYear year>×M            // 年ラベル + 全メディア合計件数のヒートバー
      </TimelineAxis>

      <TimelineLane mediaType>×N             // 1 media_type = 1行
        <TimelineLaneHeader />               // sticky left: アイコン + 和名 + 総件数
        <TimelineYearCell year>×M            // 年セル
          <TimelineDot item />×k             // 日付昇順に wrap 配置。24px サムネ
          <OverflowChip>+N</OverflowChip>    // セル内上限超過分
        </TimelineYearCell>
      </TimelineLane>
    </TimelineBoard>

    <EmptyState />                           // 条件に該当する作品が0件のとき（TimelineBoardの代わりに表示）
    <QuickViewSheet />                       // ドットクリックで開く（共通コンポーネント）
    <MediaContextMenu />                     // ドット右クリック（共通コンポーネント）
  </Content>
</AppShell>
```

グリッド構造は `TimelineBoard` を1つの CSS Grid とし、列は `[レーンヘッダ列(固定幅)] [不明列] [年列]×M`、行は `[年軸行] [レーン行]×N` とする。セルの整列をブラウザに任せることで、レーンごとの行高計算をJS側で持たなくて済む。

## 3. 表示データ / Props型

```ts
type MediaType = 'anime' | 'movie' | 'drama' | 'manga' | 'novel' | 'game' | 'academic_book' | 'paper';
type ItemStatus = 'not_started' | 'in_progress' | 'completed';
type TimelineDateField = 'release' | 'consumed' | 'any';
type TimelineDensity = 'compact' | 'default' | 'wide';

/** URLクエリと1:1で対応する画面状態 */
interface TimelineFilters {
  dateField: TimelineDateField;   // 既定 'release'
  mediaTypes: MediaType[];        // レーン表示対象。空配列 = 件数>0の全種別
  fromYear?: number;              // 未指定時は「直近20年」を実行時に算出
  toYear?: number;
  isFavorite?: boolean;
  status?: ItemStatus;
  tagId?: string;
  categoryId?: string;
  density: TimelineDensity;       // 既定 'default'
}

/** API直返し: GET /items/years（実装済み） */
interface YearCount {
  year: number;
  count: number;
  media_types: { media_type: MediaType; count: number }[];  // count降順、0件の種別は含まない
}

/** API直返し: GET /items/timeline（新規提案・未実装） */
interface TimelineEntry {
  id: string;
  media_type: MediaType;
  title: string;
  release_date: string | null;    // "YYYY-MM-DD"
  consumed_date: string | null;
  cover_image_url: string | null;
  status: ItemStatus;
  is_favorite: boolean;
  rating: number | null;
}

/** UI集約型（camelCase） */
interface TimelineBoardData {
  years: number[];                // 昇順。fromYear..toYear を欠落年も含めて連続で埋める
  lanes: TimelineLaneData[];
  maxYearCount: number;           // 年軸ヒートバーの正規化に使う（全年の合計件数の最大値）
}

interface TimelineLaneData {
  mediaType: MediaType;
  label: string;                  // 「アニメ」「映画」…
  total: number;                  // 表示年範囲内の総件数
  cells: TimelineYearCellData[];  // years と同じ長さ・同じ順序
  unknown: TimelineYearCellData;  // 日付nullの作品をまとめる「不明」セル
}

interface TimelineYearCellData {
  year: number | null;            // null = 不明列
  count: number;                  // /items/years 由来の正確な件数
  dots: TimelineDotItem[];        // /items/timeline 由来。count より少ないことがある
  overflow: number;               // count - dots.length（>0 なら +N チップを出す）
}

interface TimelineDotItem {
  id: string;
  title: string;
  coverUrl: string | null;
  dateLabel: string;              // "2019.04" / 月が不明なら "2019"
  status: ItemStatus;
  isFavorite: boolean;
}
```

`count`（`/items/years` の正確な集計）と `dots.length`（`/items/timeline` の返却上限つき実体）を**別フィールドとして持つ**のが本画面のデータ設計の要点。ヒートバーと `+N` は常に `count` を正としつつ、ドットの描画コストは上限で抑える。

## 4. 画面固有コンポーネント

| コンポーネント | 責務 |
|---|---|
| `<TimelineBoard data filters onDotClick>` | CSS Grid 本体。横スクロール、sticky の重なり制御、初期スクロール位置の設定 |
| `<TimelineAxis years maxCount density>` | sticky top の年軸。年ラベル + 合計件数ヒートバー。10年区切りを強調 |
| `<TimelineLane lane years density>` | 1 media_type の行。`TimelineLaneHeader` + `TimelineYearCell[]` |
| `<TimelineLaneHeader mediaType label total active onClick>` | sticky left。クリックでそのレーンのみに絞り込む |
| `<TimelineYearCell cell density onDotClick onOverflowClick>` | 年セル。ドットの wrap 配置と `+N` チップ |
| `<TimelineDot item onClick onContextMenu>` | 24px サムネ + ステータスリング + お気に入り枠。hoverでツールチップ |
| `<LaneVisibilityDropdown value options onChange>` | レーンに出す `media_type` の複数選択ドロップダウン |
| `<DensityToggle value onChange>` | 年カラム幅 3段の切替 |
| `<YearRangeSelect value onChange>` | 表示年範囲のプリセット選択 |

既存の共通コンポーネントを流用するもの（新規作成しない）: `FilterToolbar`（`components/shared/FilterToolbar.tsx`）、`QuickViewSheet`、`MediaContextMenu`、`EmptyState`、`MediaTypeDropdown`（`LaneVisibilityDropdown` の実装ベースとして検討）。

**チャートライブラリは導入しない。** `package.json` に recharts / d3 / visx 等は無く、本画面は CSS Grid + サムネ画像 + `div` のバーで構成できるため、依存追加の必要はない。

## 5. インタラクション仕様

### スクロール・固定

- 初期スクロール位置は**右端（最新年）**。`TimelineBoard` のマウント時に `scrollLeft = scrollWidth` を設定する
- 年軸は `sticky top-0`、レーンヘッダ列は `sticky left-0`、左上コーナーは両方 sticky（重なり順は §7）
- 年カラム幅は密度で固定（compact 88px / default 132px / wide 200px）。セル内のドットは `flex-wrap` で折り返す

### 絞り込み・遷移

- レーンヘッダ click → その `media_type` のみ表示（`filters.mediaTypes = [mediaType]`）。再クリックで解除
- 年ラベル click → `/collection/yearly?year={year}&date_field={dateField}` へ遷移
- `+N` チップ click → `/collection/yearly?year={year}&date_field={dateField}&media_type={mediaType}` へ遷移
- 件数0のレーンは既定で非表示。`LaneVisibilityDropdown` で明示的に選択された種別のみ、0件でも空レーンとして表示する
- 学術書（`academic_book`）・論文（`paper`）もレーン候補に含める（8種すべて）

### ドット

- hover → ツールチップに「タイトル / `dateLabel` / 種別」。ツールチップは `TimelineBoard` のスクロール領域外に出さない
- click → `QuickViewSheet` を開く（作品詳細への遷移は QuickView 内のリンクから）
- 右クリック → `MediaContextMenu`
- `cover_image_url` が `null` のときは `media_type` アイコン（`00_common.md` §4 の対応表）をプレースホルダとして描画する
- セル内の並び順は対象日付（`dateField` に対応するカラム）の**昇順**

### 日付が無い作品

- `release_date`（または `consumed_date`）が `null` の作品は年に配置できないため、年軸の左端に固定した**「不明」列**にまとめる
- 不明列は該当作品が1件も無ければ描画しない

### URL同期

`useSearchParams` で以下を同期する（ブラウザバック・共有リンク対応）:

```
?date_field=release&from=2006&to=2026&media_types=anime,movie&is_favorite=true&status=completed&tag_id=…&category_id=…&density=default
```

### 既知の制約

- **`release_date` に精度情報が無い。** バックエンドの `Item.release_date` は `date` 型単一で「1995年（月日不明）」と「1995-01-01」を区別できない。本画面は**年を軸に取り、月は `dateLabel` の表記に留める**ことでこの歪みが位置決めに影響しないようにしている。月を軸にする拡張を行う場合は、先にデータモデル側へ精度マーカーを追加する必要がある
- 【要確認】年カラムの仮想化（windowing）は初版では行わない。表示年範囲の既定を「直近20年」に絞ることで DOM 量を抑える方針とし、全期間表示で体感が悪化した場合に改めて検討する

## 6. API連携

### 6-1. 年軸と件数: `GET /items/years`（実装済み）

```
GET /items/years?date_field=release
```

- レスポンス: `ApiOk<YearCount[]>`（年降順、`media_types` は count 降順で0件の種別を含まない）
- **`media_type` パラメータは付けない。** 全種別の内訳が `media_types` に入るため、この1リクエストで「年 × media_type の件数マトリクス」全体が得られる
- 用途: 年軸の骨格、レーンごとの `count`、ヒートバー、`+N` の算出
- 注意: 日付が `NULL` の行は集計対象外のため、「不明」列の件数はこのレスポンスからは得られない（6-2 側で補う）
- 【要確認】このエンドポイントは実装済みだが `items.md` に記載が無かったため、本設計と合わせて `items.md` へ追記する

### 6-2. ドット実体: `GET /items/timeline`（新規提案・未実装）

既存の `GET /items` は `year` が**完全一致のみ**、`media_type` が**単一値のみ**、`limit` 上限100・キーセットページングのみであり、レーン数 × 年数ぶんのリクエストが必要になる（8種 × 20年 = 160リクエスト）。これを1リクエストに畳むための専用エンドポイントを提案する。

```
GET /items/timeline?date_field=release&from_year=2006&to_year=2026&media_type=anime,movie&limit_per_cell=20
```

- **クエリパラメータ（提案）**
  - `date_field` (string, optional, `release` / `consumed` / `any`, 既定 `release`) — 既存 `DateField` を流用
  - `from_year` / `to_year` (i32, optional) — 範囲。未指定時は全期間
  - `media_type` (string, optional) — **カンマ区切りで複数指定可**（既存エンドポイントとの差分。未指定時は全種別）
  - `is_favorite` (bool, optional) / `status` (string, optional) / `tag_id` (uuid, optional) / `category_id` (uuid, optional) — `GET /items` と同一のセマンティクス
  - `include_undated` (bool, optional, 既定 `false`) — `true` のとき対象日付が `NULL` の作品も返す（「不明」列用）
  - `limit_per_cell` (u32, optional, 既定 20, max 50) — **年 × media_type の組ごと**の返却上限。`ROW_NUMBER() OVER (PARTITION BY year, media_type ORDER BY <col>)` で切る想定
- **成功レスポンス** (200): `ApiOk<TimelineEntry[]>` — 対象日付の昇順。`TimelineEntry` は §3 参照（一覧カードに必要な最小フィールドのみ。`tags`/`categories` は含めない）
- **設計意図**: 全件を返さず年×種別ごとに上限を設けることで、所蔵件数が増えてもレスポンスサイズが `年数 × 種別数 × limit_per_cell` で上限づけられる。正確な件数は 6-1 側が持つため、UI の情報量は落ちない

【要確認】本エンドポイントは未実装。バックエンド側の実装タスクとして切り出す。それまでのフロント実装は `GET /items/years` のみで**ヒートマップ表示（件数の濃淡のみ、ドット無し）**として先行させることも可能。

### 6-3. 補助

- タグ・カテゴリの絞り込み選択肢: `GET /tags` / `GET /categories`（`TagWithCount[]` / `CategoryWithCount[]`）
- `QuickViewSheet` 内のデータ取得は既存 `useQuickViewData` に委譲する

参照: [items.md](../../backend/mediavault-api/items.md), [tags.md](../../backend/mediavault-api/tags.md), [categories.md](../../backend/mediavault-api/categories.md), [data-model.md](../../backend/mediavault-api/data-model.md)

## 7. Tailwindスタイリング上の注意

- **年カラム幅は `@theme` に足さない。** 密度によって動的に変わる値のため、`TimelineBoard` のルート要素に `style={{ '--tl-col-w': '132px' }}` の形でCSS変数として持たせ、子は `w-[var(--tl-col-w)]` で参照する。`@theme` トークンは画面横断で使う静的値のみに保つ（`00_common.md` §2 の方針）
- **sticky の重なり順**を明示的に管理する:
  - 左上コーナー: `z-30`
  - 年軸行: `z-20`
  - レーンヘッダ列: `z-10`
  - 年セル: 既定（`z-0`）
  - ツールチップ / `QuickViewSheet` はボード外のポータルに出すため対象外
- 罫線: レーン間は `border-border-soft`、年カラム間は `border-border`。10年区切り（`year % 10 === 0`）の左罫線のみ `border-text-faint` で強調する
- ドットの状態表現は既存トークンを流用する:
  - ステータスリング: `--color-status-none` / `--color-status-progress` / `--color-status-done`
  - お気に入り枠: `--color-favorite`
  - hover時の持ち上げは `bg-bg-surface-hover` + `outline-accent`
- 年軸のヒートバーは `bg-accent-soft` を下地に `bg-accent` の内側バーを `maxYearCount` で正規化した高さ（または幅）で重ねる。ダーク/ライト両テーマで `--color-accent` が切り替わるため、バー色はハードコードしない
- `TimelineBoard` は `overflow-x: auto` かつ `overflow-y: visible` にできない（CSS仕様上、片方が `auto` だともう片方は `visible` にならない）。ツールチップは `position: fixed` のポータルで描画し、セル内に閉じ込めない
- サムネ画像は `loading="lazy"` + 固定サイズ（`w-6 h-6` 等）で、画像読み込み前後のレイアウトシフトを防ぐ
