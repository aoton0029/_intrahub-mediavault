# TASK-0006: 共通UIコンポーネント実装 - TDDテストケース定義書

**機能名**: 共通UIコンポーネント（MediaCard / MediaTypeBadge / FilterBar / EmptyState / ConfirmDialog）
**タスクID**: TASK-0006
**要件名**: frontend-collection-ui
**タスクタイプ**: TDD
**フェーズ**: Phase 1 - 基盤構築
**作成日**: 2026-06-30

## 信頼性レベル凡例

- 🔵 **青信号**: EARS要件定義書・設計文書・既存実装を参考にしてほぼ推測していない
- 🟡 **黄信号**: EARS要件定義書・設計文書から妥当な推測
- 🔴 **赤信号**: EARS要件定義書・設計文書にない推測

---

## 0. 開発言語・フレームワーク

- **プログラミング言語**: TypeScript 5.7+ / React 18.3+（TSX）
  - **言語選択の理由**: プロジェクト全体が React + TypeScript で統一されており、props インターフェースを型安全に検証できる（note.md「1. 技術スタック」）。実装済み型定義 `frontend/src/types/index.ts` の `Item` 判別共用体・`MediaType` を直接利用する。
  - **テストに適した機能**: 判別共用体（Item型）・厳密な型チェックにより、テストフィクスチャの型整合性をコンパイル時に保証できる。
  - 🔵 信頼性: note.md「1. 技術スタック」、`frontend/src/types/index.ts` 実装より確実。
- **テストフレームワーク**: Vitest + @testing-library/react + @testing-library/jest-dom（環境: jsdom）
  - **フレームワーク選択の理由**: `frontend/vitest.config.ts` で Vitest（globals: true, jsdom）が設定済み、`frontend/src/test/setup.ts` で `@testing-library/jest-dom/vitest` が有効化済み（note.md「5. テスト関連情報」）。
  - **テスト実行環境**: jsdom（ブラウザDOM相当）。`yarn test` で1回実行、`yarn test:watch` でウォッチ。
  - 🔵 信頼性: vitest.config.ts, src/test/setup.ts（`import '@testing-library/jest-dom/vitest'` を実体確認）, App.test.tsx より確実。
- **ユーザー操作シミュレーション**: `@testing-library/react` の `fireEvent` または `@testing-library/user-event`（`userEvent`）
  - 🟡 信頼性: note.md「5. テスト関連情報 > テストユーティリティ・パターン」に列挙。user-event は導入推奨だが本タスクでの確定は実装時（妥当な推測）。

### テストファイル配置

| コンポーネント | 実装ファイル | テストファイル |
| --- | --- | --- |
| MediaCard | `frontend/src/components/common/MediaCard.tsx` | `frontend/src/components/common/MediaCard.test.tsx` |
| MediaTypeBadge | `frontend/src/components/common/MediaTypeBadge.tsx` | `frontend/src/components/common/MediaTypeBadge.test.tsx` |
| FilterBar | `frontend/src/components/common/FilterBar.tsx` | `frontend/src/components/common/FilterBar.test.tsx` |
| EmptyState | `frontend/src/components/common/EmptyState.tsx` | `frontend/src/components/common/EmptyState.test.tsx` |
| ConfirmDialog | `frontend/src/components/common/ConfirmDialog.tsx` | `frontend/src/components/common/ConfirmDialog.test.tsx` |

🔵 信頼性: note.md「2. 開発ルール > テスト駆動開発」「8. 注意事項・制約」より確実。`frontend/src/components/common/` は未作成（本タスクで新規作成）。

### 共通テストフィクスチャ（Item型）

`Item` は `mediaType` による判別共用体で、`id`・`title`・`status`・`isFavorite`・`source`・`createdAt`・`updatedAt`・`details` が必須（`frontend/src/types/index.ts` L118-145、`ItemBase` + 各 `Details`）。`coverImageUrl`・`isFavorite`（必須）・`status`（必須）はカード表示で使用。MediaCard テストでは以下のような最小フィクスチャを用いる想定。

```typescript
// 【テストデータ準備】: Item判別共用体の必須フィールドを満たす最小フィクスチャ生成ヘルパー
// 【初期条件設定】: mediaTypeごとにdetailsの必須配列フィールド（genreList等）を満たす
// 🔵 frontend/src/types/index.ts ItemBase/AnimeDetails の必須フィールドより
const makeAnimeItem = (overrides?: Partial<Item>): Item => ({
  id: 'item-1',
  title: 'テストアニメ',
  status: 'not_started',
  isFavorite: false,
  source: 'manual',
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: '2026-01-01T00:00:00Z',
  mediaType: 'anime',
  details: { genreList: [] },
  ...overrides,
} as Item);
```

> 注: 8種別の `details` 必須フィールドは種別ごとに異なる（anime/movie/drama: `genreList`、game: `platformList`、paper: `authorList`、manga/novel/academic_book: 必須配列なし）。`it.each` で全種別を回す境界テストでは各種別に対応した `details` を生成する。

---

## 1. MediaCard コンポーネント

**役割**: 一覧画面でアイテム1件をカード表示（title, coverImageUrl, mediaType→MediaTypeBadge, isFavorite, status）。
**Props**: `{ item: Item; onClick?: (item: Item) => void; }`
**参照**: requirements.md「2.1 MediaCard」, note.md「6. 実装対象コンポーネント > MediaCard」, `frontend/src/types/index.ts` ItemBase

### 1.1 正常系テストケース

#### TC-MC-N-01: アイテムのタイトルが表示される

- **テスト名**: MediaCard はアイテムのタイトルをレンダリングする
  - **何をテストするか**: `item.title` がカード内に表示されること
  - **期待される動作**: 渡した `item.title` の文字列が DOM に出力される
- **入力値**: `makeAnimeItem({ title: 'テストアニメ' })`
  - **入力データの意味**: 一覧表示で最も重要な識別情報がタイトル。代表的な日本語タイトルを使用
- **期待される結果**: `screen.getByText('テストアニメ')` が要素を返す（`toBeInTheDocument()`）
  - **期待結果の理由**: requirements.md「2.1 出力」で title 表示が明記されているため
- **テストの目的**: 必須フィールド title の描画確認
  - **確認ポイント**: タイトル文字列が欠落・改変されずに表示されること
- 🔵 信頼性: requirements.md「2.1 MediaCard 出力」, note.md「テスト対象: item.title が表示される」より確実

#### TC-MC-N-02: mediaType に応じた MediaTypeBadge が表示される

- **テスト名**: MediaCard は item.mediaType を MediaTypeBadge として表示する
  - **何をテストするか**: カード内に当該 mediaType のバッジ（日本語ラベル）が描画されること
  - **期待される動作**: `mediaType='anime'` のとき「アニメ」ラベルのバッジが描画される
- **入力値**: `makeAnimeItem({ mediaType: 'anime', details: { genreList: [] } })`
  - **入力データの意味**: バッジ委譲の代表ケースとして anime を使用
- **期待される結果**: `screen.getByText('アニメ')` が存在する（MediaTypeBadge 経由）
  - **期待結果の理由**: requirements.md「2.1 出力」で mediaType を MediaTypeBadge へ委譲すると明記
- **テストの目的**: MediaTypeBadge への委譲（合成）が機能することの確認
  - **確認ポイント**: MediaCard が自前でバッジを再実装せず、MediaTypeBadge を内包すること
- 🔵 信頼性: requirements.md「2.1 出力（mediaType→MediaTypeBadge）」, note.md より確実

#### TC-MC-N-03: onClick 指定時にカードクリックで onClick(item) が呼ばれる

- **テスト名**: MediaCard はクリック時に onClick を当該 item 付きで呼ぶ
  - **何をテストするか**: カードクリックで `onClick` コールバックが `item` を引数に1回呼ばれること
  - **期待される動作**: クリックイベントで `onClick(item)` が発火する
- **入力値**: `onClick = vi.fn()`、`makeAnimeItem()`、`fireEvent.click(card)` または `userEvent.click`
  - **入力データの意味**: 一覧→詳細遷移の起点となるクリック導線を検証
- **期待される結果**: `onClick` が `toHaveBeenCalledTimes(1)` かつ `toHaveBeenCalledWith(item)`
  - **期待結果の理由**: requirements.md「2.1 出力」で onClick 指定時 `onClick(item)` を呼ぶと明記
- **テストの目的**: クリックコールバックの発火と引数の正確性確認
  - **確認ポイント**: 呼び出し回数が1回、引数が当該 item そのものであること
- 🔵 信頼性: requirements.md「2.1」, note.md「テスト対象: onClick がカードクリックで呼び出される」より確実

#### TC-MC-N-04: coverImageUrl 指定時に画像（cover）が表示される

- **テスト名**: MediaCard は coverImageUrl をカバー画像として表示する
  - **何をテストするか**: `item.coverImageUrl` が `<img>` の `src`（または背景）として描画されること
  - **期待される動作**: 画像要素の参照先 URL が渡した値になる
- **入力値**: `makeAnimeItem({ coverImageUrl: 'https://example.com/cover.jpg' })`
  - **入力データの意味**: 一覧でのビジュアル識別に使われるカバー画像の代表 URL
- **期待される結果**: `screen.getByRole('img')` の `src`（または該当要素）が `'https://example.com/cover.jpg'` を含む
  - **期待結果の理由**: requirements.md「2.1 出力」で coverImageUrl（画像）表示が明記
- **テストの目的**: カバー画像の描画確認
  - **確認ポイント**: 渡した URL が正しく `src` に反映されること（alt は title 等を期待、🟡）
- 🟡 信頼性: requirements.md「2.1」で画像表示は明記だが、img role/alt 等の具体実装は推測

#### TC-MC-N-05: isFavorite=true のときお気に入り表示がされる

- **テスト名**: MediaCard は isFavorite=true でお気に入り状態を視覚表示する
  - **何をテストするか**: `isFavorite=true` のときお気に入りを示す要素（アイコン/ラベル等）が描画されること
  - **期待される動作**: お気に入りインジケータが表示状態になる
- **入力値**: `makeAnimeItem({ isFavorite: true })`
  - **入力データの意味**: お気に入りON状態の代表ケース
- **期待される結果**: お気に入りを示す要素（例: `aria-label` や `data-favorite` 属性、アイコン）が存在する
  - **期待結果の理由**: requirements.md「2.1 出力」で isFavorite 表示が明記
- **テストの目的**: お気に入り状態の視覚反映確認
  - **確認ポイント**: ON 状態が判別可能な形でレンダリングされること
- 🟡 信頼性: requirements.md「2.1」で表示は明記だが、具体的表現（アイコン/属性）は実装時決定の推測

#### TC-MC-N-06: status が視覚表示される

- **テスト名**: MediaCard は item.status を表示する
  - **何をテストするか**: `status`（not_started / in_progress / completed）に応じた表示がされること
  - **期待される動作**: 各 status を示すラベルまたは属性が描画される
- **入力値**: `makeAnimeItem({ status: 'in_progress' })`
  - **入力データの意味**: 進行中状態の代表ケース（3状態のうち中間値）
- **期待される結果**: status を示す要素（ラベル文字列または `data-status="in_progress"` 等）が存在する
  - **期待結果の理由**: requirements.md「2.1 出力」で status 表示が明記
- **テストの目的**: 視聴・読了ステータスの表示確認
  - **確認ポイント**: 渡した status と表示が対応していること
- 🟡 信頼性: requirements.md「2.1」で表示は明記だが、表示形式（ラベル文言/属性）は推測

### 1.2 異常系テストケース

#### TC-MC-E-01: onClick 未指定でもクリックしてエラーにならない

- **テスト名**: onClick 省略時にカードクリックしても例外が発生しない
  - **エラーケースの概要**: 任意 prop である onClick を渡さずにクリックした場合
  - **エラー処理の重要性**: onClick はオプショナルであり、未指定時に落ちると一覧表示自体が破綻する
- **入力値**: `makeAnimeItem()`（onClick なし）、`fireEvent.click(card)`
  - **不正な理由**: 不正ではなく「任意 prop 省略 + クリック」という未定義動作の境界
  - **実際の発生シナリオ**: クリック不要なプレビュー用途等で onClick を渡さない場面
- **期待される結果**: クリックしても例外を throw せず、レンダリングが維持される
  - **エラーメッセージの内容**: エラーメッセージは発生しない（無害化）
  - **システムの安全性**: undefined を呼び出す `TypeError` が起きないこと
- **テストの目的**: オプショナルコールバックの安全なガード確認
  - **品質保証の観点**: 任意 prop 省略時のクラッシュ耐性を保証
- 🟡 信頼性: requirements.md「4.2 MediaCard（onClickなし）」から妥当な推測

#### TC-MC-E-02: coverImageUrl 欠落時にプレースホルダ表示で描画が破綻しない

- **テスト名**: coverImageUrl が undefined でもエラーなく描画する
  - **エラーケースの概要**: 任意フィールド coverImageUrl が存在しないアイテム
  - **エラー処理の重要性**: 手動登録アイテムや API 画像なしアイテムでカードが壊れないこと
- **入力値**: `makeAnimeItem({ coverImageUrl: undefined })`
  - **不正な理由**: 不正ではなく任意フィールド欠落。`src=undefined` で壊れ得る境界
  - **実際の発生シナリオ**: source='manual' で画像未設定のアイテム表示時
- **期待される結果**: title 等は表示され、画像欠落はプレースホルダ等で吸収され例外が出ない
  - **エラーメッセージの内容**: エラーは発生しない
  - **システムの安全性**: 画像欠落でカード全体が消えない・例外で落ちない
- **テストの目的**: 任意画像フィールド欠落時の描画堅牢性確認
  - **品質保証の観点**: 実データのばらつきに対する耐性を保証
- 🟡 信頼性: requirements.md「4.2 MediaCard（任意項目欠落）」から妥当な推測

### 1.3 境界値テストケース

#### TC-MC-B-01: 8種別すべての mediaType で MediaCard がエラーなく描画される

- **テスト名**: MediaCard は8種別すべての Item をエラーなく描画する
  - **境界値の意味**: 判別共用体の全分岐（anime〜paper）が網羅される境界
  - **境界値での動作保証**: details の型が種別ごとに異なっても描画が成立すること
- **入力値**: 8種別それぞれの最小 Item（`it.each` で `['anime','movie','drama','manga','novel','game','academic_book','paper']` を反復。各種別の details 必須フィールドを満たす）
  - **境界値選択の根拠**: MediaType の取りうる全値（`frontend/src/types/index.ts` L12-20、media-type-accent.ts の8キー）
  - **実際の使用場面**: 混在コレクションの一覧表示で全種別が並ぶ場面
- **期待される結果**: いずれの種別でも `getByText(item.title)` が成立し、例外が発生しない
  - **境界での正確性**: 各種別の details 必須フィールド差異を吸収して描画
  - **一貫した動作**: 全種別で title 描画とバッジ委譲が一貫
- **テストの目的**: 全媒体種別に対する描画堅牢性確認
  - **堅牢性の確認**: academic_book / paper を含む全分岐の網羅
- 🔵 信頼性: `frontend/src/types/index.ts` MediaType（8種別）, requirements.md「4.2 8種別網羅」より確実

#### TC-MC-B-02: タイトルが空文字でも描画が破綻しない

- **テスト名**: title が空文字でも MediaCard はクラッシュしない
  - **境界値の意味**: 必須文字列フィールドの最小値（空文字）
  - **境界値での動作保証**: 空文字でもカード枠が描画されること
- **入力値**: `makeAnimeItem({ title: '' })`
  - **境界値選択の根拠**: 文字列必須フィールドの下限（長さ0）
  - **実際の使用場面**: データ不整合や未入力タイトルの防御的描画
- **期待される結果**: 例外が発生せず、カードコンテナがレンダリングされる
  - **境界での正確性**: 空文字を許容しつつ描画破綻しない
  - **一貫した動作**: 空・非空で描画構造が一貫
- **テストの目的**: 文字列境界での堅牢性確認
  - **堅牢性の確認**: 空入力に対する防御
- 🟡 信頼性: `frontend/src/types/index.ts` で title は必須 string。空文字の扱いは設計文書に明記なし（妥当な推測）

---

## 2. MediaTypeBadge コンポーネント

**役割**: MediaType を受け取り、対応する色（getMediaTypeAccentClass）と日本語ラベルでバッジ表示。
**Props**: `{ mediaType: MediaType; }`
**参照**: requirements.md「2.2 MediaTypeBadge」, `frontend/src/lib/media-type-accent.ts`, note.md「6 > MediaTypeBadge」

### 2.1 正常系テストケース

#### TC-MB-N-01: anime のラベルとアクセントクラスが適用される

- **テスト名**: MediaTypeBadge は anime に対し「アニメ」ラベルと text-accent-anime を適用する
  - **何をテストするか**: 日本語ラベルとアクセントカラークラスの双方が描画されること
  - **期待される動作**: 「アニメ」テキストを持つ要素に `text-accent-anime` クラスが付く
- **入力値**: `mediaType='anime'`
  - **入力データの意味**: 代表的な media_type。8種別の先頭
- **期待される結果**: `screen.getByText('アニメ')` が存在し、その要素（または親バッジ）が `className` に `text-accent-anime` を含む
  - **期待結果の理由**: requirements.md「2.2 出力」+ `frontend/src/lib/media-type-accent.ts` L12（anime→text-accent-anime）
- **テストの目的**: ラベル変換とアクセント色適用の両立確認
  - **確認ポイント**: getMediaTypeAccentClass の戻り値が実際に className へ反映されること
- 🔵/🟡 信頼性: media-type-accent.ts（クラス`text-accent-anime`は実体確認・確実）+ 🟡（日本語ラベル「アニメ」は requirements.md「2.2」の妥当な推測。Green/Refactor で確定）

#### TC-MB-N-02: アクセントクラスはハードコードでなく getMediaTypeAccentClass 経由で適用される

- **テスト名**: MediaTypeBadge は media-type-accent.ts のクラスを適用する
  - **何をテストするか**: 各種別のクラス文字列が media-type-accent.ts の定義と一致すること
  - **期待される動作**: movie→`text-accent-movie`, manga→`text-accent-manga` 等が反映される
- **入力値**: `mediaType='movie'`（および `'manga'`）
  - **入力データの意味**: anime 以外でクラスマッピングが正しいか確認する代表値
- **期待される結果**: badge 要素の className に `text-accent-movie`（manga の場合 `text-accent-manga`）が含まれる
  - **期待結果の理由**: 制約「色値をハードコードしない／getMediaTypeAccentClass から取得」（requirements.md 3章）
- **テストの目的**: アクセント色取得経路の正当性確認
  - **確認ポイント**: 色のハードコードでなく関数戻り値に依存していること
- 🔵 信頼性: media-type-accent.ts L11-20（実体確認）, requirements.md「3. 制約条件」より確実

### 2.2 異常系テストケース

#### TC-MB-E-01: 想定外の文字列を渡してもクラッシュしない（型外入力の防御）

- **テスト名**: 未定義の mediaType 値でもクラッシュしない
  - **エラーケースの概要**: 型を逸脱した文字列（例: `'unknown'`）が実行時に渡るケース
  - **エラー処理の重要性**: API データ不整合等で MediaType 外の値が来ても画面全体が落ちない
- **入力値**: `mediaType={'unknown' as MediaType}`
  - **不正な理由**: MediaType の8値に含まれず、`getMediaTypeAccentClass` が `MEDIA_TYPE_ACCENT_CLASS[key]` で `undefined` を返し得る
  - **実際の発生シナリオ**: バックエンド種別追加・型ズレ・破損データ受信時
- **期待される結果**: 例外を throw せずに描画される（クラスが undefined/空でもバッジ枠は出る）
  - **エラーメッセージの内容**: ランタイム例外を出さない
  - **システムの安全性**: undefined クラスで `React` が落ちないこと
- **テストの目的**: 型外入力に対する防御的描画の確認
  - **品質保証の観点**: 実行時の予期せぬ値に対するクラッシュ耐性
- 🔴 信頼性: 設計文書に明記なし。一般的な防御的UI観点からの推測（実装方針により skip 可）

### 2.3 境界値テストケース

#### TC-MB-B-01: 8種別すべてが異なるアクセントクラスでエラーなく描画される

- **テスト名**: MediaTypeBadge は8種別すべてを描画し、それぞれ対応クラスを適用する
  - **境界値の意味**: MediaType 全列挙値（anime, movie, drama, manga, novel, game, academic_book, paper）の網羅
  - **境界値での動作保証**: 全分岐でラベル描画とクラス適用が成立する
- **入力値**: `it.each` で8種別を反復。各種別と期待クラスの対応表（media-type-accent.ts 実体より）:
  - anime→`text-accent-anime`, movie→`text-accent-movie`, drama→`text-accent-drama`, manga→`text-accent-manga`, novel→`text-accent-novel`, game→`text-accent-game`, academic_book→`text-accent-academic-book`, paper→`text-accent-paper`
  - **境界値選択の根拠**: `frontend/src/lib/media-type-accent.ts` `MEDIA_TYPE_ACCENT_CLASS` の8キー全件（L11-20、実体確認）
  - **実際の使用場面**: 全種別が混在する一覧での種別判別
- **期待される結果**: 各反復で例外なく描画され、badge の className に対応するアクセントクラスが含まれる
  - **境界での正確性**: academic_book→`text-accent-academic-book`（アンダースコア→ハイフン変換）も正確
  - **一貫した動作**: 全8種別で「ラベル + アクセントクラス」の構造が一貫
- **テストの目的**: 全種別網羅とクラスマッピング正確性の確認
  - **堅牢性の確認**: 命名変換（academic_book→academic-book）を含む全分岐の検証
- 🔵 信頼性: media-type-accent.ts L11-20（8キー・クラス名を実体確認）, requirements.md「4.2 8種別網羅」より確実

---

## 3. FilterBar コンポーネント（枠のみ）

**役割**: 絞り込みUI の器（コンテナ）。詳細UIは TASK-0010（Phase 2）。
**Props**: `{ children?: React.ReactNode; }`
**参照**: requirements.md「2.3 FilterBar」, note.md「6 > FilterBar」

### 3.1 正常系テストケース

#### TC-FB-N-01: children が正しくレンダリングされる

- **テスト名**: FilterBar は children をそのまま描画する
  - **何をテストするか**: 渡した子要素がコンテナ内に出力されること
  - **期待される動作**: `<FilterBar><div>filter</div></FilterBar>` で「filter」が描画される
- **入力値**: `children = <button>絞り込み</button>`（または `<div data-testid="child" />`）
  - **入力データの意味**: 後続タスクで差し込む絞り込みUIの代理。任意の ReactNode を代表
- **期待される結果**: `screen.getByText('絞り込み')`（または `getByTestId('child')`）が存在する
  - **期待結果の理由**: requirements.md「2.3 出力（children をそのまま内包）」より
- **テストの目的**: 器コンポーネントとしての children パススルー確認
  - **確認ポイント**: children が改変されず描画されること
- 🔵 信頼性: requirements.md「2.3」, note.md「テスト対象: children が正しくレンダリングされる」より確実

### 3.2 異常系テストケース

#### TC-FB-E-01: children に複数要素を渡しても全て描画される

- **テスト名**: FilterBar は複数の children を全て描画する
  - **エラーケースの概要**: 単一でなく複数子要素（配列）を渡すケース
  - **エラー処理の重要性**: 実運用では複数のフィルタUI要素を並べるため、複数子の描画が必須
- **入力値**: `children = [<span key="a">A</span>, <span key="b">B</span>]`
  - **不正な理由**: 不正ではないが、単一子のみ前提だと取りこぼす境界
  - **実際の発生シナリオ**: media_type/タグ/status 等、複数のフィルタ要素を並置する場面
- **期待される結果**: 「A」「B」両方が描画される
  - **エラーメッセージの内容**: エラーは発生しない
  - **システムの安全性**: 複数子で描画が欠落しないこと
- **テストの目的**: 複数 children のパススルー確認
  - **品質保証の観点**: 器として一般的な children を漏れなく受け入れる保証
- 🟡 信頼性: requirements.md「2.3」から妥当な推測（複数子は ReactNode の一般仕様）

### 3.3 境界値テストケース

#### TC-FB-B-01: children 未指定でもコンテナのみエラーなく描画される

- **テスト名**: FilterBar は children 省略時にコンテナのみ描画する
  - **境界値の意味**: 任意 prop children の最小（未指定 / undefined）
  - **境界値での動作保証**: 子なしでも器が成立すること
- **入力値**: `<FilterBar />`（children なし）
  - **境界値選択の根拠**: children はオプショナル（`children?`）。未指定が下限
  - **実際の使用場面**: 初期表示やフィルタ要素未配置時の器のみ描画
- **期待される結果**: 例外なくコンテナ要素がレンダリングされる（`container` が空でない）
  - **境界での正確性**: undefined children を安全に無視
  - **一貫した動作**: 子あり・なしで器構造が一貫
- **テストの目的**: 任意 children 省略時の堅牢性確認
  - **堅牢性の確認**: 空コンテナでもクラッシュしない
- 🟡 信頼性: requirements.md「2.3」「4.2」より妥当な推測

---

## 4. EmptyState コンポーネント

**役割**: アイテム0件時の空状態表示（メッセージ + 任意のアクション導線）。
**Props**: `{ message: string; actionLabel?: string; onAction?: () => void; }`
**参照**: requirements.md「2.4 EmptyState」, requirements.md EDGE-101, note.md「6 > EmptyState」

### 4.1 正常系テストケース

#### TC-ES-N-01: message が表示される

- **テスト名**: EmptyState は message を表示する
  - **何をテストするか**: 必須 prop message がそのまま描画されること
  - **期待される動作**: 「コレクションがありません」等のメッセージが表示される
- **入力値**: `message='コレクションがありません'`
  - **入力データの意味**: EDGE-101 で例示される代表的な空状態メッセージ
- **期待される結果**: `screen.getByText('コレクションがありません')` が存在する
  - **期待結果の理由**: requirements.md「2.4 出力」+ EDGE-101 より
- **テストの目的**: 空状態メッセージの描画確認
  - **確認ポイント**: message 文字列が改変されず表示されること
- 🔵 信頼性: requirements.md「2.4」, EDGE-101, note.md より確実

#### TC-ES-N-02: actionLabel + onAction 指定時にアクションボタンが表示される

- **テスト名**: EmptyState は actionLabel 指定時にアクションボタンを表示する
  - **何をテストするか**: `actionLabel` + `onAction` が両方指定されたときボタンが描画されること
  - **期待される動作**: ボタン要素に actionLabel のテキストが表示される
- **入力値**: `message='...'`, `actionLabel='アイテムを追加'`, `onAction=vi.fn()`
  - **入力データの意味**: 追加導線（EDGE-101「追加画面への導線」）の代表ケース
- **期待される結果**: `screen.getByRole('button', { name: 'アイテムを追加' })` が存在する
  - **期待結果の理由**: requirements.md「2.4 出力（actionLabel + onAction 両方指定時のみボタンを表示）」より
- **テストの目的**: アクション導線ボタンの表示確認
  - **確認ポイント**: actionLabel がボタンの可視ラベルとして反映されること
- 🟡 信頼性: requirements.md「2.4」+ EDGE-101「追加画面への導線」より妥当な推測

#### TC-ES-N-03: アクションボタンクリックで onAction が呼ばれる

- **テスト名**: EmptyState はアクションボタンクリックで onAction を呼ぶ
  - **何をテストするか**: ボタンクリックで `onAction` が1回呼ばれること
  - **期待される動作**: クリックイベントで `onAction()` が発火する
- **入力値**: `actionLabel='アイテムを追加'`, `onAction=vi.fn()`、`userEvent.click(button)`
  - **入力データの意味**: 追加画面遷移トリガとしてのクリック導線を検証
- **期待される結果**: `onAction` が `toHaveBeenCalledTimes(1)`
  - **期待結果の理由**: requirements.md「2.4 出力（クリックで onAction を呼ぶ）」より
- **テストの目的**: アクションコールバックの発火確認
  - **確認ポイント**: クリック1回で正確に1回呼ばれること
- 🟡 信頼性: requirements.md「2.4」, note.md「テスト対象: ボタンクリックで onAction が呼ばれる」より妥当な推測

### 4.2 異常系テストケース

#### TC-ES-E-01: actionLabel 指定で onAction 未指定時もクリックでクラッシュしない

- **テスト名**: actionLabel のみ指定（onAction 省略）でクリックしても例外が出ない
  - **エラーケースの概要**: ラベルだけ渡しコールバックを忘れた不整合な使い方
  - **エラー処理の重要性**: onAction が任意のため、未指定時のクリックで落ちると致命的
- **入力値**: `actionLabel='追加'`（`onAction` なし）、`userEvent.click(button)`（ボタンが描画される実装の場合）
  - **不正な理由**: ラベルとコールバックがペア前提だが onAction 欠落
  - **実際の発生シナリオ**: 呼び出し側の prop 渡し漏れ
- **期待される結果**: クリックしても `TypeError`（undefined 呼び出し）が発生しない
  - **エラーメッセージの内容**: エラーは発生しない
  - **システムの安全性**: undefined コールバックへの安全なガード
- **テストの目的**: 任意コールバック未指定時の安全性確認
  - **品質保証の観点**: prop 不整合に対するクラッシュ耐性
- 🟡 信頼性: requirements.md「2.4 入出力の関係性（両方指定時のみボタン表示）」からの妥当な推測。実装が「actionLabel と onAction の両方が揃った時のみボタン描画」とする場合は本ケースは「ボタンが描画されない」確認に読み替える

### 4.3 境界値テストケース

#### TC-ES-B-01: actionLabel/onAction 省略時はメッセージのみ表示しボタンを描画しない

- **テスト名**: EmptyState は action 省略時にボタンを描画しない
  - **境界値の意味**: 任意 props（actionLabel/onAction）の最小（いずれも未指定）
  - **境界値での動作保証**: メッセージのみの最小構成が成立すること
- **入力値**: `message='コレクションがありません'`（action 系なし）
  - **境界値選択の根拠**: actionLabel/onAction はオプショナル。両方未指定が下限構成
  - **実際の使用場面**: 単純な空状態（導線不要）の表示
- **期待される結果**: メッセージは表示され、`screen.queryByRole('button')` が `null`（ボタン非描画）
  - **境界での正確性**: action 未指定時にボタンを出さない
  - **一貫した動作**: action あり/なしで描画要素が適切に切り替わる
- **テストの目的**: 任意 action 省略時の条件付きレンダリング確認
  - **堅牢性の確認**: 余計なボタンを描画しないこと
- 🟡 信頼性: requirements.md「4.2 EmptyState（actionなし）: メッセージのみ表示」より妥当な推測

#### TC-ES-B-02: message が空文字でもエラーなく描画される

- **テスト名**: message 空文字でも EmptyState はクラッシュしない
  - **境界値の意味**: 必須文字列 message の最小値（空文字）
  - **境界値での動作保証**: 空文字でも空状態枠が描画される
- **入力値**: `message=''`
  - **境界値選択の根拠**: 必須 string の下限（長さ0）
  - **実際の使用場面**: メッセージ未設定時の防御的描画
- **期待される結果**: 例外なくコンテナが描画される
  - **境界での正確性**: 空文字を許容しつつ破綻しない
  - **一貫した動作**: 空・非空で描画構造が一貫
- **テストの目的**: 文字列境界での堅牢性確認
  - **堅牢性の確認**: 空メッセージへの防御
- 🟡 信頼性: requirements で message は必須 string。空文字扱いは明記なし（妥当な推測）

---

## 5. ConfirmDialog コンポーネント

**役割**: 削除等の確認ダイアログ（controlled）。shadcn/ui の Dialog をベースにする。
**Props**: `{ open: boolean; title: string; description?: string; onConfirm: () => void; onCancel: () => void; confirmLabel?: string; cancelLabel?: string; }`
**参照**: requirements.md「2.5 ConfirmDialog」, requirements.md REQ-007, note.md「6 > ConfirmDialog」

### 5.1 正常系テストケース

#### TC-CD-N-01: open=true で title と description が表示される

- **テスト名**: ConfirmDialog は open=true で title/description を表示する
  - **何をテストするか**: 表示状態でタイトルと本文が描画されること
  - **期待される動作**: `title`・`description` 文字列が DOM に出力される
- **入力値**: `open=true`, `title='削除しますか？'`, `description='この操作は取り消せません'`, `onConfirm/onCancel=vi.fn()`
  - **入力データの意味**: 削除確認（REQ-007）の代表ケース
- **期待される結果**: `getByText('削除しますか？')` と `getByText('この操作は取り消せません')` が存在する
  - **期待結果の理由**: requirements.md「2.5 出力（open=true で title/description 表示）」より
- **テストの目的**: 表示状態でのコンテンツ描画確認
  - **確認ポイント**: タイトル・本文の双方が表示されること
- 🟡 信頼性: requirements.md「2.5」より妥当な推測（shadcn Dialog ベース）

#### TC-CD-N-02: 確認ボタンクリックで onConfirm が呼ばれる

- **テスト名**: ConfirmDialog は確認ボタンクリックで onConfirm を呼ぶ
  - **何をテストするか**: 確認ボタンクリックで `onConfirm` が1回呼ばれること
  - **期待される動作**: クリックで `onConfirm()` が発火する
- **入力値**: `open=true`, `confirmLabel='削除'`, `onConfirm=vi.fn()`、`userEvent.click(confirmButton)`
  - **入力データの意味**: 削除実行トリガとなる確認操作を検証
- **期待される結果**: `onConfirm` が `toHaveBeenCalledTimes(1)`
  - **期待結果の理由**: requirements.md「2.5 出力（確認ボタンで onConfirm を呼ぶ）」より
- **テストの目的**: 確認コールバックの発火確認
  - **確認ポイント**: 確認ボタンと onConfirm が正しく結線されていること
- 🟡 信頼性: requirements.md「2.5」, note.md「テスト対象: 確認ボタンクリックで onConfirm」より妥当な推測

#### TC-CD-N-03: キャンセルボタンクリックで onCancel が呼ばれる

- **テスト名**: ConfirmDialog はキャンセルボタンクリックで onCancel を呼ぶ
  - **何をテストするか**: キャンセルボタンクリックで `onCancel` が1回呼ばれること
  - **期待される動作**: クリックで `onCancel()` が発火する
- **入力値**: `open=true`, `cancelLabel='キャンセル'`, `onCancel=vi.fn()`、`userEvent.click(cancelButton)`
  - **入力データの意味**: 操作中止トリガとなるキャンセル操作を検証
- **期待される結果**: `onCancel` が `toHaveBeenCalledTimes(1)`、かつ `onConfirm` は呼ばれない（`not.toHaveBeenCalled()`）
  - **期待結果の理由**: requirements.md「2.5 出力（キャンセルボタンで onCancel を呼ぶ）」より
- **テストの目的**: キャンセルコールバックの発火と確認系の非発火確認
  - **確認ポイント**: キャンセル操作で onConfirm が誤って呼ばれないこと
- 🟡 信頼性: requirements.md「2.5」, note.md「テスト対象: キャンセルボタンクリックで onCancel」より妥当な推測

#### TC-CD-N-04: confirmLabel/cancelLabel 指定時にラベルがボタンへ反映される

- **テスト名**: ConfirmDialog は指定された confirmLabel/cancelLabel をボタンに表示する
  - **何をテストするか**: 任意ラベル prop がボタンの可視テキストになること
  - **期待される動作**: confirmLabel='削除', cancelLabel='やめる' がそれぞれボタンに出る
- **入力値**: `open=true`, `confirmLabel='削除'`, `cancelLabel='やめる'`
  - **入力データの意味**: 操作に応じたカスタムラベル（削除文脈）の代表ケース
- **期待される結果**: `getByRole('button', { name: '削除' })` と `getByRole('button', { name: 'やめる' })` が存在する
  - **期待結果の理由**: requirements.md「2.5 入力（confirmLabel/cancelLabel 任意）」より
- **テストの目的**: カスタムラベルの反映確認
  - **確認ポイント**: 渡したラベルがそのままボタン名になること
- 🟡 信頼性: requirements.md「2.5」より妥当な推測

### 5.2 異常系テストケース

#### TC-CD-E-01: 連続クリックでも onConfirm が呼ばれる（多重発火の検証）

- **テスト名**: 確認ボタンを2回クリックすると onConfirm が2回呼ばれる
  - **エラーケースの概要**: ダブルクリック等で確認が多重発火する状況
  - **エラー処理の重要性**: 削除など破壊的操作での多重実行挙動を把握する必要がある
- **入力値**: `open=true`, `onConfirm=vi.fn()`、`userEvent.click(confirmButton)` を2回
  - **不正な理由**: 不正ではないが、破壊操作で多重発火しうる境界
  - **実際の発生シナリオ**: ユーザーの素早い連打
- **期待される結果**: `onConfirm` が `toHaveBeenCalledTimes(2)`（本コンポーネントは多重発火抑制を持たず素直に伝播することを明示）
  - **エラーメッセージの内容**: エラーは発生しない
  - **システムの安全性**: コンポーネント単体は伝播に徹し、冪等性は呼び出し側責務であることを確認
- **テストの目的**: 多重クリック時のコールバック挙動の明確化
  - **品質保証の観点**: 破壊操作の多重実行責務境界を明示
- 🔴 信頼性: 設計文書に明記なし。controlled コンポーネントの一般挙動からの推測（実装方針により期待値変更可）

### 5.3 境界値テストケース

#### TC-CD-B-01: open=false でダイアログ内容が描画されない

- **テスト名**: ConfirmDialog は open=false で内容を描画しない
  - **境界値の意味**: 表示制御 boolean の境界（true/false）
  - **境界値での動作保証**: 非表示時にタイトル・ボタンが DOM に出ないこと
- **入力値**: `open=false`, `title='削除しますか？'`, `onConfirm/onCancel=vi.fn()`
  - **境界値選択の根拠**: open の取りうる2値のうち非表示側。requirements.md「4.2 open=false」
  - **実際の使用場面**: ダイアログ閉状態（初期状態・閉じた後）
- **期待される結果**: `screen.queryByText('削除しますか？')` が `null`（非描画）
  - **境界での正確性**: open=false でコンテンツが描画されない（shadcn Dialog の挙動準拠）
  - **一貫した動作**: open の true/false で表示・非表示が切り替わる
- **テストの目的**: 非表示状態の描画抑制確認
  - **堅牢性の確認**: 閉状態でコンテンツが漏れ表示されないこと
- 🟡 信頼性: requirements.md「2.5」「4.2 ConfirmDialog: open=false」より妥当な推測

#### TC-CD-B-02: confirmLabel/cancelLabel 省略時にデフォルトラベルが表示される

- **テスト名**: ConfirmDialog はラベル省略時にデフォルト文言を表示する
  - **境界値の意味**: 任意ラベル props の最小（未指定 → デフォルト適用）
  - **境界値での動作保証**: 未指定でも確認/キャンセル両ボタンが描画される
- **入力値**: `open=true`, `title='...'`, `onConfirm/onCancel=vi.fn()`（label 系なし）
  - **境界値選択の根拠**: confirmLabel/cancelLabel はオプショナル。未指定時のデフォルト動作を検証
  - **実際の使用場面**: 標準的な確認ダイアログ（ラベル指定省略）
- **期待される結果**: 確認・キャンセル相当のボタンが2つ描画される（デフォルト文言、例:「OK」「キャンセル」等は実装時確定）
  - **境界での正確性**: ラベル未指定でもボタンが欠落しない
  - **一貫した動作**: ラベル指定有無でボタン数が一致（2ボタン）
- **テストの目的**: デフォルトラベルのフォールバック確認
  - **堅牢性の確認**: 任意 prop 未指定時のフォールバック動作
- 🟡 信頼性: requirements.md「2.5 入出力の関係性（既定文言は実装時決定）」より妥当な推測。具体的文言はアサーションを緩く（ボタン2個存在）とする

---

## 6. テストケース実装時の日本語コメント指針（共通テンプレート）

各テストファイルでは以下の構造で日本語コメントを付与する（note.md「2. 開発ルール」準拠）。

```typescript
import { describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MediaCard } from './MediaCard'
import type { Item } from '@/types'

// 【共通フィクスチャ】: Item判別共用体の必須フィールドを満たす最小 anime Item 生成ヘルパー
const makeAnimeItem = (overrides?: Partial<Item>): Item => ({
  id: 'item-1', title: 'テストアニメ', status: 'not_started', isFavorite: false,
  source: 'manual', createdAt: '2026-01-01T00:00:00Z', updatedAt: '2026-01-01T00:00:00Z',
  mediaType: 'anime', details: { genreList: [] }, ...overrides,
} as Item)

describe('MediaCard', () => {
  it('アイテムのタイトルを表示する', () => {
    // 【テスト目的】: item.title がカードに表示されることを確認する
    // 【テスト内容】: makeAnimeItem で生成した Item を渡し、title 文字列の描画を検証する
    // 【期待される動作】: 渡した title がそのまま DOM に出力される
    // 🔵 requirements.md「2.1 MediaCard 出力」より

    // 【テストデータ準備】: 必須フィールドを満たす最小 anime Item を用意する
    // 【初期条件設定】: details.genreList=[] 等、判別共用体の必須フィールドを満たす
    const item = makeAnimeItem({ title: 'テストアニメ' })

    // 【実際の処理実行】: MediaCard をレンダリングする
    // 【処理内容】: render() で jsdom 上に MediaCard をマウントする
    render(<MediaCard item={item} />)

    // 【結果検証】: title 文字列が DOM に存在するか検証する
    // 【期待値確認】: getByText('テストアニメ') が要素を返すこと
    // 【品質保証】: 一覧表示の最重要情報であるタイトル描画を保証する
    // 【検証項目】: タイトル文字列の表示
    // 🔵 requirements.md「2.1」より
    expect(screen.getByText('テストアニメ')).toBeInTheDocument() // 【確認内容】: タイトルが欠落せず表示されることを確認
  })
})
```

クリック検証の例（コールバック発火）:

```typescript
it('カードクリックで onClick が item 付きで呼ばれる', async () => {
  // 【テスト目的】: onClick コールバックの発火と引数の正確性を確認する 🔵
  // 【テスト内容】: onClick=vi.fn() を渡しクリック、呼び出し回数と引数を検証する
  const onClick = vi.fn() // 【テストデータ準備】: 呼び出し記録用のモック関数
  const item = makeAnimeItem()
  render(<MediaCard item={item} onClick={onClick} />)

  // 【実際の処理実行】: カード要素をクリックする
  await userEvent.click(screen.getByText(item.title))

  // 【検証項目】: 呼び出し回数1回・引数が item であること
  expect(onClick).toHaveBeenCalledTimes(1) // 【確認内容】: 過不足なく1回だけ発火
  expect(onClick).toHaveBeenCalledWith(item) // 【確認内容】: 当該 item が引数に渡る
})
```

`it.each` による8種別網羅の例（MediaTypeBadge）:

```typescript
const cases: [MediaType, string, string][] = [
  ['anime', 'text-accent-anime', 'アニメ'],
  ['movie', 'text-accent-movie', '映画'],
  ['drama', 'text-accent-drama', 'ドラマ'],
  ['manga', 'text-accent-manga', '漫画'],
  ['novel', 'text-accent-novel', '小説'],
  ['game', 'text-accent-game', 'ゲーム'],
  ['academic_book', 'text-accent-academic-book', '専門書'],
  ['paper', 'text-accent-paper', '論文'],
]
it.each(cases)('mediaType=%s で %s クラスが適用される', (mediaType, accentClass) => {
  // 【検証項目】: getMediaTypeAccentClass 由来のクラスが className に含まれること
  const { container } = render(<MediaTypeBadge mediaType={mediaType} />)
  expect(container.querySelector(`.${accentClass}`)).not.toBeNull() // 🔵 media-type-accent.ts より
})
```

> 日本語ラベル（「映画」「漫画」等）は requirements に明記がない 🟡 推測のため、ラベル文言のアサーションは Green/Refactor で確定する。クラス名のアサーション（🔵）を主軸にすると安定する。

---

## 7. テストケース一覧（網羅サマリー）

| ID | コンポーネント | 分類 | 概要 | 信頼性 |
| --- | --- | --- | --- | --- |
| TC-MC-N-01 | MediaCard | 正常系 | title が表示される | 🔵 |
| TC-MC-N-02 | MediaCard | 正常系 | mediaType の MediaTypeBadge が表示される | 🔵 |
| TC-MC-N-03 | MediaCard | 正常系 | クリックで onClick(item) が呼ばれる | 🔵 |
| TC-MC-N-04 | MediaCard | 正常系 | coverImageUrl が画像表示される | 🟡 |
| TC-MC-N-05 | MediaCard | 正常系 | isFavorite=true でお気に入り表示 | 🟡 |
| TC-MC-N-06 | MediaCard | 正常系 | status が表示される | 🟡 |
| TC-MC-E-01 | MediaCard | 異常系 | onClick 未指定でクリックしても落ちない | 🟡 |
| TC-MC-E-02 | MediaCard | 異常系 | coverImageUrl 欠落で破綻しない | 🟡 |
| TC-MC-B-01 | MediaCard | 境界値 | 8種別すべてエラーなく描画 | 🔵 |
| TC-MC-B-02 | MediaCard | 境界値 | title 空文字で破綻しない | 🟡 |
| TC-MB-N-01 | MediaTypeBadge | 正常系 | anime ラベル+アクセントクラス適用 | 🔵/🟡 |
| TC-MB-N-02 | MediaTypeBadge | 正常系 | getMediaTypeAccentClass 経由でクラス適用 | 🔵 |
| TC-MB-E-01 | MediaTypeBadge | 異常系 | 型外値でクラッシュしない | 🔴 |
| TC-MB-B-01 | MediaTypeBadge | 境界値 | 8種別すべて異なるクラスで描画 | 🔵 |
| TC-FB-N-01 | FilterBar | 正常系 | children が描画される | 🔵 |
| TC-FB-E-01 | FilterBar | 異常系 | 複数 children を全て描画 | 🟡 |
| TC-FB-B-01 | FilterBar | 境界値 | children 未指定でコンテナのみ描画 | 🟡 |
| TC-ES-N-01 | EmptyState | 正常系 | message が表示される | 🔵 |
| TC-ES-N-02 | EmptyState | 正常系 | actionLabel+onAction でボタン表示 | 🟡 |
| TC-ES-N-03 | EmptyState | 正常系 | ボタンクリックで onAction が呼ばれる | 🟡 |
| TC-ES-E-01 | EmptyState | 異常系 | onAction 未指定でクリックしても落ちない | 🟡 |
| TC-ES-B-01 | EmptyState | 境界値 | action 省略時はボタン非描画 | 🟡 |
| TC-ES-B-02 | EmptyState | 境界値 | message 空文字で破綻しない | 🟡 |
| TC-CD-N-01 | ConfirmDialog | 正常系 | open=true で title/description 表示 | 🟡 |
| TC-CD-N-02 | ConfirmDialog | 正常系 | 確認ボタンで onConfirm が呼ばれる | 🟡 |
| TC-CD-N-03 | ConfirmDialog | 正常系 | キャンセルボタンで onCancel が呼ばれる | 🟡 |
| TC-CD-N-04 | ConfirmDialog | 正常系 | カスタムラベルがボタンに反映 | 🟡 |
| TC-CD-E-01 | ConfirmDialog | 異常系 | 連続クリックで onConfirm が2回 | 🔴 |
| TC-CD-B-01 | ConfirmDialog | 境界値 | open=false で内容非描画 | 🟡 |
| TC-CD-B-02 | ConfirmDialog | 境界値 | ラベル省略でデフォルト表示 | 🟡 |

**合計**: 30件（正常系15 / 異常系5 / 境界値10）

### コンポーネント別内訳

| コンポーネント | 正常系 | 異常系 | 境界値 | 小計 |
| --- | --- | --- | --- | --- |
| MediaCard | 6 | 2 | 2 | 10 |
| MediaTypeBadge | 2 | 1 | 1 | 4 |
| FilterBar | 1 | 1 | 1 | 3 |
| EmptyState | 3 | 1 | 2 | 6 |
| ConfirmDialog | 4 | 1 | 2 | 7 |
| **合計** | **16** | **6** | **8** | **30** |

### 信頼性レベル分布

- 🔵 青信号: 9件（30%）— title 表示・onClick・MediaTypeBadge 委譲・8種別網羅・children パススルー・accent クラス・message 表示
- 🟡 黄信号: 19件（63%）— レイアウト詳細・日本語ラベル・EmptyState/ConfirmDialog の挙動・任意 prop 省略時の堅牢性
- 🔴 赤信号: 2件（7%）— MediaTypeBadge 型外値防御・ConfirmDialog 多重発火（設計文書に明記なし、実装方針により調整可）

---

## 8. 要件定義との対応関係

- **参照した機能概要**: requirements.md「1. 機能の概要」（5コンポーネントの役割・配置）
- **参照した入力・出力仕様**: requirements.md「2. 入力・出力の仕様」（2.1〜2.5 各 props と出力）
- **参照した制約条件**: requirements.md「3. 制約条件」（配置先 components/common・shadcn/ui ベース・cva/cn・getMediaTypeAccentClass・Vitest+Testing Library・jsdom）
- **参照した使用例**: requirements.md「4. 想定される使用例」（基本パターン・データフロー・エッジケース・エラーケース）
- **参照したEARS要件**: REQ-001（一覧カード表示→MediaCard）, REQ-002/REQ-003（絞り込み→FilterBar枠）, REQ-007（削除→ConfirmDialog）, EDGE-101（0件→EmptyState）
- **参照した設計文書 / 実装**: architecture.md「コンポーネント粒度」, `frontend/src/types/index.ts`（Item 判別共用体, MediaType, ItemStatus, ItemBase）, `frontend/src/lib/media-type-accent.ts`（getMediaTypeAccentClass, MEDIA_TYPE_ACCENT_CLASS 全8キー）, `frontend/src/components/ui/button.tsx`（cva/cn 参考）, `frontend/src/test/setup.ts`・`frontend/vitest.config.ts`・`frontend/src/App.test.tsx`（テスト基盤）

---

## 9. 品質判定

```
✅ 高品質:
- テストケース分類: 正常系（16）・異常系（6）・境界値（8）を5コンポーネント全てで網羅
- 期待値定義: 各ケースで具体的な入力値・期待アサーション（getByText/getByRole/toHaveBeenCalled 等）を明示
- 技術選択: TypeScript + Vitest + @testing-library/react（jsdom）で確定（設定ファイル・型定義・media-type-accent.ts を実体確認済み）
- 実装可能性: 依存（media-type-accent.ts, utils.ts, shadcn Button, Item型）は既存実装で確認済み。Dialog/Badge は shadcn 追加（npx shadcn@latest add badge dialog）で導入可能
- 信頼性レベル: 🔵9 / 🟡19 / 🔴2。🔴2件は設計文書外の防御的ケースで、実装方針により調整可能（TDDの阻害要因なし）
```

---

## 10. 次のステップ

次のお勧めステップ: `/tsumiki:tdd-red frontend-collection-ui TASK-0006` でRedフェーズ（失敗テスト作成）を開始します。
