// テストファイル: frontend/src/design-tokens.test.ts
// 【目的】: TASK-0001「デザイントークンの再定義（index.css）」の完了条件を検証する
// 【方針】: index.css のCSS変数値自体はjsdomで実カスケード計算できないため、
//          ファイル内容（テキスト）を直接読み込み、完了条件（TASK-0001.md）が
//          要求するトークン名・値がすべて定義されているかを静的に検証する。
//          （design-tokens-testcases.md TC-04, TC-05 に対応）
// 【リファクタ】: Refactorフェーズで、各テストに繰り返し現れる正規表現組み立てを
//              tokenPattern() ヘルパーに共通化し、可読性・保守性を向上（振る舞いは変更なし） 🔵
import { describe, test, expect, beforeAll } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const INDEX_CSS_PATH = resolve(__dirname, './index.css');

let cssContent: string;

beforeAll(() => {
  // 【テスト前準備】: 対象CSSファイルをテキストとして読み込み、以降の全テストで再利用する
  // 【環境初期化】: ファイルシステムから毎回最新の内容を取得するため、キャッシュせず読み直す
  cssContent = readFileSync(INDEX_CSS_PATH, 'utf-8');
});

/**
 * 【ヘルパー関数】: `--トークン名: 値` 形式のCSS変数宣言を検出する正規表現を組み立てる
 * 【再利用性】: 本ファイル内の全テストケースで、CSS変数の存在・値検証に共通して利用する
 * 【単一責任】: 正規表現の組み立てのみを担当し、値の間の空白差異を吸収する
 * @param name - トークン名（先頭の `--` は含めない、例: 'bg-app'）
 * @param valuePattern - 期待する値にマッチする正規表現文字列（呼び出し側で必要なエスケープ済みのもの）
 * @returns トークン宣言を検出するための正規表現
 */
function tokenPattern(name: string, valuePattern: string): RegExp {
  // 【処理効率化】: 各テストで個別に書いていた `new RegExp` 組み立てを一箇所に集約 🔵
  return new RegExp(`--${name}:\\s*${valuePattern}`);
}

describe('design-tokens (TASK-0001: index.css デザイントークン再定義)', () => {
  test('TC-04-1: 背景色系トークン（--bg-app, --bg-sidebar, --bg-surface, --bg-surface-hover, --bg-input）が定義されている', () => {
    // 【テスト目的】: _shared.css準拠の新規背景色トークンがすべて追加されていることを確認する
    // 【テスト内容】: index.cssのテキスト内容に各トークン宣言（変数名+値）が含まれるかを検証する
    // 【期待される動作】: Greenフェーズの実装により、以下のトークンが全て定義されている
    // 🔵 信頼性レベル: design-tokens-requirements.md 出力仕様、design-tokens-testcases.md TC-04に明記

    expect(cssContent).toMatch(tokenPattern('bg-app', '#1e1e1e')); // 【確認内容】: メイン背景色トークンが_shared.css準拠の値で定義されていることを確認
    expect(cssContent).toMatch(tokenPattern('bg-sidebar', '#161616')); // 【確認内容】: サイドバー背景色トークンが定義されていることを確認
    expect(cssContent).toMatch(tokenPattern('bg-surface', '#262626')); // 【確認内容】: サーフェス背景色トークンが新しい値(#262626)で定義されていることを確認（旧値#1a1d23からの置換）
    expect(cssContent).toMatch(tokenPattern('bg-surface-hover', '#2c2c2c')); // 【確認内容】: ホバー時サーフェス背景色トークンが定義されていることを確認
    expect(cssContent).toMatch(tokenPattern('bg-input', '#1c1c1c')); // 【確認内容】: 入力欄背景色トークンが定義されていることを確認
  });

  test('TC-04-2: 境界線系トークン（--border, --border-soft）が新しい値で定義されている', () => {
    // 【テスト目的】: 境界線色トークンが_shared.css準拠の値に置換されていることを確認する
    // 【テスト内容】: --borderが新値#383838であること、--border-softが新規追加されていることを検証
    // 🔵 信頼性レベル: design-tokens-requirements.md 3節（--border衝突対応方針）、testcases.md TC-04に明記

    expect(cssContent).toMatch(tokenPattern('border', '#383838')); // 【確認内容】: 境界線色トークンが_shared.css準拠の16進値に置換されていることを確認
    expect(cssContent).toMatch(tokenPattern('border-soft', '#2e2e2e')); // 【確認内容】: 補助境界線色トークンが新規追加されていることを確認
  });

  test('TC-04-3: 文字色系トークン（--text-primary, --text-muted, --text-faint）が定義されている', () => {
    // 【テスト目的】: 文字色トークンが_shared.css準拠の新しい値・変数名に置換されていることを確認する
    // 【テスト内容】: --text-primaryの新値、--text-muted/--text-faintの新規変数名を検証
    // 🔵 信頼性レベル: design-tokens-requirements.md 出力仕様に明記

    expect(cssContent).toMatch(tokenPattern('text-primary', '#dcddde')); // 【確認内容】: 主要文字色が新しい値に置換されていることを確認
    expect(cssContent).toMatch(tokenPattern('text-muted', '#8a8a8d')); // 【確認内容】: 旧--text-secondaryに代わる--text-mutedが新規定義されていることを確認
    expect(cssContent).toMatch(tokenPattern('text-faint', '#5c5c5f')); // 【確認内容】: 新規の淡色文字トークンが定義されていることを確認
  });

  test('TC-04-4: 単一アクセント色トークン（--accent, --accent-strong, --accent-soft）が定義されている', () => {
    // 【テスト目的】: モックアップ準拠の単一アクセント色（紫系）が定義されていることを確認する
    // 【テスト内容】: --accentが新しい紫系16進値であること、強調色・薄色バリエーションが追加されていることを検証
    // 🔵 信頼性レベル: design-tokens-requirements.md 出力仕様「単一アクセント色」に明記

    expect(cssContent).toMatch(tokenPattern('brand-accent', '#8b6cf6')); // 【確認内容】: 単一アクセント色（紫系）が_shared.css準拠の値で定義されていることを確認（TASK-0002でshadcn--accentとの名前衝突回避のため--brand-accentへリネーム）
    expect(cssContent).toMatch(tokenPattern('accent-strong', '#a48bf8')); // 【確認内容】: 強調用アクセント色が新規定義されていることを確認
    expect(cssContent).toMatch(tokenPattern('accent-soft', String.raw`rgba\(139,\s*108,\s*246,\s*0\.15\)`)); // 【確認内容】: 薄色バリエーション（rgba）が新規定義されていることを確認
  });

  test('TC-04-5: ステータス色トークン（--favorite, --status-progress, --status-done, --status-none, --danger）が定義されている', () => {
    // 【テスト目的】: お気に入り・進行状況・エラー等を表すステータス色一式が定義されていることを確認する
    // 【テスト内容】: 5つのステータス色トークンの存在と値を検証
    // 🔵 信頼性レベル: design-tokens-requirements.md 出力仕様「ステータス色」に明記

    expect(cssContent).toMatch(tokenPattern('favorite', '#e0a85a')); // 【確認内容】: お気に入りステータス色が定義されていることを確認
    expect(cssContent).toMatch(tokenPattern('status-progress', '#5aa9e0')); // 【確認内容】: 進行中ステータス色が定義されていることを確認
    expect(cssContent).toMatch(tokenPattern('status-done', '#5ac98a')); // 【確認内容】: 完了ステータス色が定義されていることを確認
    expect(cssContent).toMatch(tokenPattern('status-none', '#6b6b6e')); // 【確認内容】: 未着手ステータス色が定義されていることを確認
    expect(cssContent).toMatch(tokenPattern('danger', '#e0615a')); // 【確認内容】: エラー・危険操作を表す色が定義されていることを確認
  });

  test('TC-04-6: フォント系トークン（--font-ui, --font-display, --font-mono）が定義されている', () => {
    // 【テスト目的】: _shared.css準拠のフォントファミリー変数が定義されていることを確認する
    // 【テスト内容】: Inter / Source Serif 4 / JetBrains Mono を含むフォント変数の存在を検証
    // 🔵 信頼性レベル: design-tokens-requirements.md 出力仕様「フォント」に明記

    expect(cssContent).toMatch(tokenPattern('font-ui', `['"]Inter['"]`)); // 【確認内容】: UI用フォント変数がInterを含んで定義されていることを確認
    expect(cssContent).toMatch(tokenPattern('font-display', `['"]Source Serif 4['"]`)); // 【確認内容】: 見出し用フォント変数がSource Serif 4を含んで定義されていることを確認
    expect(cssContent).toMatch(tokenPattern('font-mono', `['"]JetBrains Mono['"]`)); // 【確認内容】: 等幅フォント変数がJetBrains Monoを含んで定義されていることを確認
  });

  test('TC-04-7: レイアウト・角丸トークン（--sidebar-w, --properties-w, --radius, --radius-shadcn）が定義されている', () => {
    // 【テスト目的】: レイアウト幅トークンの新規追加と、--radiusの名前衝突回避（--radius-shadcnへのリネーム）を確認する
    // 【テスト内容】: サイドバー幅・プロパティパネル幅の新規定義、--radius:6pxへの置換、--radius-shadcn:0.625remの共存を検証
    // 🔵🟡 信頼性レベル: レイアウト幅は🔵（要件定義書に明記）、--radius-shadcnへのリネームは🟡（note.md/testcases.mdでの確定方針）

    expect(cssContent).toMatch(tokenPattern('sidebar-w', '232px')); // 【確認内容】: サイドバー幅トークンが新規定義されていることを確認
    expect(cssContent).toMatch(tokenPattern('properties-w', '300px')); // 【確認内容】: プロパティパネル幅トークンが新規定義されていることを確認
    expect(cssContent).toMatch(tokenPattern('radius', '6px')); // 【確認内容】: 角丸トークンが_shared.css準拠の6pxに置換されていることを確認
    expect(cssContent).toMatch(tokenPattern('radius-shadcn', String.raw`0\.625rem`)); // 【確認内容】: shadcn用の旧--radius値が--radius-shadcnという別名で共存していることを確認（名前衝突回避）
  });

  test('TC-04-8: media_type別アクセントカラー8色が変更されず維持されている（REQ-002）', () => {
    // 【テスト目的】: デザイントークン再定義後もmedia_type別アクセントカラーが一切変更されていないことを確認する
    // 【テスト内容】: 8色の変数名・値がすべて既存実装と一致することを検証（回帰確認）
    // 🔵 信頼性レベル: REQ-002、design-tokens-requirements.md・testcases.md TC-08に明記

    expect(cssContent).toMatch(tokenPattern('accent-anime', '#f97316')); // 【確認内容】: アニメ用アクセント色が変更されていないことを確認
    expect(cssContent).toMatch(tokenPattern('accent-movie', '#3b82f6')); // 【確認内容】: 映画用アクセント色が変更されていないことを確認
    expect(cssContent).toMatch(tokenPattern('accent-manga', '#ec4899')); // 【確認内容】: 漫画用アクセント色が変更されていないことを確認
    expect(cssContent).toMatch(tokenPattern('accent-novel', '#8b5cf6')); // 【確認内容】: 小説用アクセント色が変更されていないことを確認
    expect(cssContent).toMatch(tokenPattern('accent-game', '#10b981')); // 【確認内容】: ゲーム用アクセント色が変更されていないことを確認
    expect(cssContent).toMatch(tokenPattern('accent-drama', '#ef4444')); // 【確認内容】: ドラマ用アクセント色が変更されていないことを確認
    expect(cssContent).toMatch(tokenPattern('accent-academic-book', '#14b8a6')); // 【確認内容】: 学術書用アクセント色が変更されていないことを確認
    expect(cssContent).toMatch(tokenPattern('accent-paper', '#64748b')); // 【確認内容】: 論文用アクセント色が変更されていないことを確認
  });

  test('TC-04-9: 旧トークン（--bg-base, 旧--text-secondary, --border-default等）が:rootから削除されている', () => {
    // 【テスト目的】: _shared.css準拠置換に伴い、置換対象の旧トークン宣言が完全に削除されていることを確認する
    // 【テスト内容】: --bg-base, --text-secondary, --border-defaultの宣言が存在しないことを検証
    // 🔵 信頼性レベル: design-tokens-testcases.md 0.5節(B)、TC-04確認ポイントに明記

    expect(cssContent).not.toMatch(tokenPattern('bg-base', '#0f1115')); // 【確認内容】: 旧背景色トークン--bg-baseの宣言が削除されていることを確認
    expect(cssContent).not.toMatch(tokenPattern('text-secondary', '#9ca3af')); // 【確認内容】: 旧文字色トークン--text-secondaryの宣言が削除されていることを確認（--text-mutedに置換）
    expect(cssContent).not.toMatch(tokenPattern('border-default', '#2d313a')); // 【確認内容】: 旧境界線色トークン--border-defaultの宣言が削除されていることを確認
  });

  test('TC-05: @theme inline内の--radius参照が--radius-shadcnに追随修正されている（未定義参照リスク対策）', () => {
    // 【テスト目的】: --radiusの値が0.625rem→6pxに変わることで、@theme inlineのradius-sm〜radius-4xl計算式が
    //              意図せず6pxベースに巻き込まれる事故を防ぐため、参照先がvar(--radius-shadcn)に
    //              書き換えられていることを確認する
    // 【テスト内容】: @theme inlineブロック内の--radius-lg等の計算式にvar(--radius-shadcn)が使われているかを検証
    // 🟡 信頼性レベル: design-tokens-testcases.md 0.5節(A)の確定方針に基づく（資料に厳密な命名指定なし）

    expect(cssContent).toMatch(tokenPattern('radius-lg', String.raw`var\(--radius-shadcn\)`)); // 【確認内容】: shadcn用角丸計算の基準変数が--radius-shadcnに追随修正されていることを確認
    expect(cssContent).not.toMatch(/--radius-lg:\s*var\(--radius\)\s*;/); // 【確認内容】: 旧参照（var(--radius)のみ）が残っていないことを確認（新--radius:6pxへの巻き込み事故防止）
  });
});

describe('design-tokens (TASK-0002: Tailwind @theme連携とshadcn oklchトークンの上書き)', () => {
  test('TC-01: shadcn oklch系トークンが_shared.css準拠のCSS変数参照に置き換わっている（REQ-402）', () => {
    // 【テスト目的】: --primary, --card, --popover等のshadcn標準トークンが、oklchハードコード値ではなく
    //              _shared.css由来のトークン変数を参照するよう上書きされていることを確認する
    // 🔵 信頼性レベル: TASK-0002.md 完了条件・実装詳細に明記

    expect(cssContent).toMatch(tokenPattern('background', String.raw`var\(--bg-app\)`));
    expect(cssContent).toMatch(tokenPattern('foreground', String.raw`var\(--text-primary\)`));
    expect(cssContent).toMatch(tokenPattern('card', String.raw`var\(--bg-surface\)`));
    expect(cssContent).toMatch(tokenPattern('popover', String.raw`var\(--bg-surface\)`));
    expect(cssContent).toMatch(tokenPattern('secondary', String.raw`var\(--bg-surface-hover\)`));
    expect(cssContent).toMatch(tokenPattern('muted', String.raw`var\(--bg-surface-hover\)`));
    expect(cssContent).toMatch(tokenPattern('destructive', String.raw`var\(--danger\)`));
    expect(cssContent).toMatch(tokenPattern('input', String.raw`var\(--bg-input\)`));
    expect(cssContent).toMatch(tokenPattern('ring', String.raw`var\(--accent-strong\)`));
  });

  test('TC-02: shadcnの--accentと_shared.cssの単一アクセント色が--brand-accentへのリネームで名前衝突を回避している', () => {
    // 【テスト目的】: shadcnの--accent（hover背景用ニュートラル）と_shared.cssの単一アクセント色（紫系）が
    //              同名で衝突しないよう、後者が--brand-accentという別名で定義されていることを確認する
    // 🟡 信頼性レベル: TASK-0002.md「shadcn oklchトークンの上書き」節の設計判断より

    expect(cssContent).toMatch(tokenPattern('brand-accent', '#8b6cf6'));
    expect(cssContent).toMatch(tokenPattern('primary', String.raw`var\(--brand-accent\)`));
    expect(cssContent).toMatch(tokenPattern('accent', String.raw`var\(--accent-soft\)`));
  });

  test('TC-03: .darkクラスブロックが:rootと同一のトークン参照に揃えられている（方針B）', () => {
    // 【テスト目的】: .darkクラスの有無に関わらず同一の見た目になるよう、.darkブロックが
    //              :rootと同じCSS変数参照（ハードコードoklchではなく）を使用していることを確認する
    // 🟡 信頼性レベル: TASK-0002.md「.darkクラスブロックの整合」節（方針B採用）より

    const darkBlockMatch = cssContent.match(/\.dark\s*\{([\s\S]*?)\n\}/);
    expect(darkBlockMatch).not.toBeNull();
    const darkBlock = darkBlockMatch ? darkBlockMatch[1] : '';

    expect(darkBlock).toMatch(new RegExp(String.raw`--background:\s*var\(--bg-app\)`));
    expect(darkBlock).toMatch(new RegExp(String.raw`--card:\s*var\(--bg-surface\)`));
    expect(darkBlock).toMatch(new RegExp(String.raw`--primary:\s*var\(--brand-accent\)`));
    expect(darkBlock).toMatch(new RegExp(String.raw`--border:\s*var\(--border\)`));
  });

  test('TC-04: media_type別アクセントカラーのTailwindユーティリティマッピングが変更されていない（REQ-002回帰確認）', () => {
    // 【テスト目的】: shadcn --accent系トークンの上書きが、@theme inline内のmedia_type専用
    //              --color-accent-*マッピングに影響を与えていないことを確認する
    // 🔵 信頼性レベル: TASK-0002.md 完了条件「media_type別アクセントカラーが変更されずそのまま維持」より

    expect(cssContent).toMatch(tokenPattern('color-accent-anime', String.raw`var\(--accent-anime\)`));
    expect(cssContent).toMatch(tokenPattern('color-accent-movie', String.raw`var\(--accent-movie\)`));
    expect(cssContent).toMatch(tokenPattern('color-accent-paper', String.raw`var\(--accent-paper\)`));
  });
});
