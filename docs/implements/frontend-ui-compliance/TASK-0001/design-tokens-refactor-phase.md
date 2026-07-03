# TASK-0001: デザイントークンの再定義（index.css）- TDD Refactorフェーズ

## 1. 実施したリファクタリング

TASK-0001のスコープ（`frontend/src/index.css` のトークン定義・`tsconfig.app.json`・`design-tokens.test.ts`）に閉じた範囲で以下を改善した。TASK-0002のスコープ（Tailwind `@theme` 連携、shadcn oklchトークン上書き、`.dark`/`prefers-color-scheme`ブロック整理）には踏み込んでいない。

### 1-1. `frontend/src/index.css`: `@import` 順序警告の解消 🔵

- **問題**: Greenフェーズの`yarn build`で「`@import rules must precede all rules aside from @charset and @layer statements`」という警告が発生していた。
- **原因**: `@import "@fontsource-variable/geist";` はビルド時に実際の`@font-face`ルール群へ展開される。そのため、ソース上でその後方に置かれていたGoogle Fonts用`@import url(...)`が、バンドル後のCSSでは「展開済みルールの後に続く@import」という扱いになり、CSS仕様（`@import`は`@charset`/`@layer`以外の全ルールに先行しなければならない）に違反していた。
- **改善内容**: Google Fonts用`@import url(...)`を最上部（`@import "tailwindcss";`等より前）に移動。他のnpm経由`@import`（`tailwindcss`, `tw-animate-css`, `shadcn/tailwind.css`, `@fontsource-variable/geist`）も同様にビルド後に実ルールへ展開されるため、外部URLの`@import`を先頭に置くことで警告要因を解消した。
- **確認**: `yarn build`実行時にCSS関連の警告が出力されなくなったことを確認済み（機能面の変更なし、フォント読み込み自体はimport文であるため順序変更の影響なし）。

```css
/* 【フォント導入】: _shared.css準拠のGoogle Fonts CDN方式でInter/Source Serif 4/JetBrains Monoを導入する 🟡 */
/* 【import順序改善】: 本importを他の@importより前に配置し、CSSバンドル時の
   「@import rules must precede all rules」警告を解消する。
   `@fontsource-variable/geist`はビルド時に実際の@font-faceルールへ展開されるため、
   後方に置くと本importがそのルール群より後続になり警告の原因になっていた 🔵 */
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=Source+Serif+4:opsz,wght@8..60,400;8..60,600&family=JetBrains+Mono:wght@400;500&display=swap');
@import "tailwindcss";
@import "tw-animate-css";
@import "shadcn/tailwind.css";
@import "@fontsource-variable/geist";
```

### 1-2. `frontend/src/design-tokens.test.ts`: テストコードのDRY化・コメント更新 🔵

- **問題**: 各テストケースで `new RegExp` 相当の正規表現リテラルを個別に手書きしており、同一パターン（`--トークン名:\s*値`）の重複が多かった。また、Red→Green移行後もコメントに「現時点では未実装のため失敗する」という**Redフェーズ当時の説明**が残っており、Refactorフェーズ時点の実態（実装済み・全件成功）と齟齬があった。
- **改善内容**:
  - `tokenPattern(name, valuePattern)` ヘルパー関数を追加し、CSS変数宣言を検出する正規表現の組み立てロジックを一箇所に集約。全10テストケースの`expect(cssContent).toMatch(...)`呼び出しをこのヘルパー経由に統一した。
  - 各テストの「期待される動作」コメントのうち、Redフェーズ限定の「現時点では...失敗する」という記述を削除し、Greenフェーズ実装後の実態（トークンが定義済みであること）を説明する記述に更新。
  - アサーション自体（マッチ対象の正規表現パターン・値）は一切変更しておらず、機能的な振る舞いの変更はない。
- **確認**: リファクタ後も同一の10テストが全件成功することを確認済み。

```typescript
/**
 * 【ヘルパー関数】: `--トークン名: 値` 形式のCSS変数宣言を検出する正規表現を組み立てる
 * 【再利用性】: 本ファイル内の全テストケースで、CSS変数の存在・値検証に共通して利用する
 * 【単一責任】: 正規表現の組み立てのみを担当し、値の間の空白差異を吸収する
 * @param name - トークン名（先頭の `--` は含めない、例: 'bg-app'）
 * @param valuePattern - 期待する値にマッチする正規表現文字列（呼び出し側で必要なエスケープ済みのもの）
 * @returns トークン宣言を検出するための正規表現
 */
function tokenPattern(name: string, valuePattern: string): RegExp {
  return new RegExp(`--${name}:\\s*${valuePattern}`);
}
```

### 1-3. `frontend/tsconfig.app.json`

- Greenフェーズで追加した `"types": [..., "node"]` は、`design-tokens.test.ts` が `node:fs`/`node:path`/`__dirname` を使用するために必要な最小限の追随修正であり、Refactorフェーズでの追加変更は不要と判断した（既に適切な状態）。

## 2. セキュリティレビュー

- 変更対象は静的なCSS変数定義・テストコード・tsconfigのみであり、ユーザー入力の処理・外部データの取り扱い・認証認可ロジックを含まない。
- Google Fonts CDNの`@import url(...)`は既存のGreenフェーズから継続利用しており、リファクタで新たな外部リソース読み込みは追加していない。
- 重大なセキュリティ上の懸念は発見されなかった。

## 3. パフォーマンスレビュー

- `@import`順序の変更はビルド後の最終CSSバイト数・読み込みリソース数に影響を与えない（宣言順序のみの変更、リソースの追加・削除なし）。実際にビルド後の`dist/assets/index-*.css`のサイズはリファクタ前後でほぼ同一（53.50kB→53.61kB、リビルドによるハッシュ変更起因の差異のみ）。
- `tokenPattern()`ヘルパーの導入によるテスト実行時間への影響は無視できるレベル（正規表現オブジェクトの生成コストは`describe`ブロック内の各テストで数回発生するのみ）。
- 重大なパフォーマンス課題は発見されなかった。

## 4. テスト結果

### 新規テスト（リファクタ後）
```
yarn test design-tokens.test.ts
```
結果: `Test Files 1 passed (1)` / `Tests 10 passed (10)` — 全件成功（リファクタ前と同数・同内容のテストが継続成功）

### 既存テスト（全体回帰、リファクタ後）
```
yarn test
```
結果: `Test Files 22 passed (22)` / `Tests 192 passed (192)` — 全件成功（回帰なし）

### ビルド確認（リファクタ後）
```
yarn build
```
結果: 成功（`tsc -b && vite build` エラーなし）。**Greenフェーズで確認されていたCSS `@import`順序警告は解消**され、CSS関連の警告は出力されなくなった（残る警告は既存の別件であるJSチャンクサイズ警告のみで、本タスクのスコープ外）。

## 5. 品質判定

- テスト結果: ✅ 全て継続成功（10/10、192/192）
- セキュリティ: ✅ 重大な脆弱性なし
- パフォーマンス: ✅ 重大な性能課題なし、CSSサイズへの実質的な影響なし
- リファクタ目標: ✅ 達成（`@import`順序警告の解消、テストコードのDRY化・コメント整合性向上）
- コード品質: ✅ 適切なレベル（`index.css`: 変更後も1ファイル内で完結する範囲、`design-tokens.test.ts`: 145行程度でヘルパー導入により重複削減、500行制限内）
- ドキュメント: ✅ 本ファイルおよびGreenフェーズ記録に完了を反映済み

**総合判定: ✅ 高品質**
