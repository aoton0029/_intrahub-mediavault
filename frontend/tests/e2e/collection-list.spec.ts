/**
 * TASK-0023: E2Eテスト（一覧系） - GeneralMediaPage collection browsing
 *
 * 対象: frontend/src/pages/GeneralMediaPage/index.tsx（ルート `/general-media`）
 * 参照:
 *   - docs/implements/collection-browsing/TASK-0023/requirements.md
 *   - docs/implements/collection-browsing/TASK-0023/testcases.md
 *
 * 実装済み機能（TASK-0011/0012/0013）に対する回帰防止E2Eテスト。
 * 全ケースは page.route() でネットワークをモックし、実バックエンドに依存しない。
 */
import { test, expect } from '@playwright/test';
import {
  mockItems,
  mockItemsForScroll,
  mockFilteredItems,
  mockSearchItems,
  mockItemsRoute,
  mockItemsRouteByPage,
  mockItemsRouteByQuery,
  mockItemsRouteErrorThenSuccess,
  mockItemsRouteAlwaysError,
} from './fixtures/items';

// 【設定定数】: TanStack Queryの既定リトライ（3回・指数バックオフ）が尽きて
//   isError が確定するまでに要する最大待機時間の目安値
// 【調整可能性】: この値は QueryClient のデフォルトretry設定（本タスクの対象外である
//   src/lib配下のQueryClient定義）に依存する。将来retry回数や遅延ミスを短縮する場合は、
//   その変更に合わせて本定数も見直すこと。プロダクションコードは本リファクタ対象外のため
//   ここでは値を変更せず、依存関係を明示するコメント・定数化のみ行う（Greenフェーズ申し送り事項）
// 🟡 信頼性レベル: green-phase.md §7の申し送りに基づく妥当な整理（挙動・値は変更なし）
const RETRY_EXHAUST_TIMEOUT_MS = 15000;

test.describe('collection-browsing GeneralMediaPage E2E', () => {
  // ==============================
  // 1. 正常系テストケース
  // ==============================

  test('TC-001-01: 一覧表示で複数アイテムがカード描画される', async ({ page }) => {
    // 【テスト目的】: page.goto('/general-media') で GET /api/v1/items が発火し、返却された Item 配列が
    //   .media-card として描画されることを確認する
    // 【テスト内容】: 20件のItemを返すモックを登録し、初期表示のカード件数を検証する
    // 【期待される動作】: page=1・limit=20 のリクエストが送られ、20件のカードが描画される
    // 🔵 信頼性レベル: testcases.md TC-001-01 準拠（実コード GeneralMediaPage/index.tsx 照合済み）

    // 【テストデータ準備】: 20件のItemを含む成功レスポンスを route で注入する
    // 【初期条件設定】: page.route は goto より前に登録し確実にインターセプトする
    await mockItemsRoute(page, { items: mockItems, total: 20 });

    // 【実際の処理実行】: 一覧画面へ遷移し items リクエスト完了を待つ
    const resP = page.waitForResponse((r) => r.url().includes('/api/v1/items'));
    await page.goto('/general-media');
    await resP;

    // 【結果検証】: カード件数が20件であること
    await expect(page.locator('.media-card')).toHaveCount(20); // 【確認内容】: 返却件数分のカードが描画される
    // 【期待値確認】: 各カードにタイトルとバッジが存在することも確認する
    await expect(page.locator('.media-card').first().locator('.title')).toBeVisible(); // 【確認内容】: カードにタイトル要素が描画されている
    await expect(page.locator('.media-card').first().locator('.badge')).toBeVisible(); // 【確認内容】: カードにメディア種別バッジが描画されている
  });

  test('TC-001-02: お気に入りアイテムに★マークが表示される', async ({ page }) => {
    // 【テスト目的】: is_favorite:true の Item を含むモックで、対応カードに .fav が描画されることを確認する
    // 【テスト内容】: 20件中3件が is_favorite:true のモックを登録し、.fav の件数を検証する
    // 【期待される動作】: favoriteのカードのみ .fav（★）を持ち、その総数が3件と一致する
    // 🔵 信頼性レベル: testcases.md TC-001-02 準拠（MediaCard.tsx の is_favorite 分岐描画を確認済み）

    // 【テストデータ準備】: mockItems は先頭3件が is_favorite:true になるよう定義済み
    await mockItemsRoute(page, { items: mockItems, total: 20 });

    // 【実際の処理実行】: 一覧画面へ遷移しレスポンスを待つ
    const resP = page.waitForResponse((r) => r.url().includes('/api/v1/items'));
    await page.goto('/general-media');
    await resP;

    // 【結果検証】: .fav（★）の件数が3件であること
    // 【確認ポイント】: .fav は aria-hidden のため CSS ロケータで取得する（getByText不可）
    await expect(page.locator('.media-card .fav')).toHaveCount(3); // 【確認内容】: is_favorite:trueのカード数と★マーク数が一致する
  });

  test('TC-101-01: お気に入り＋特定タグ(#SF)のAND絞り込みがURL・APIクエリに反映される', async ({ page }) => {
    // 【テスト目的】: 2チップ選択で URL と API クエリに is_favorite=true と tag_id=SF が
    //   両方付与され、絞り込み結果のみ表示されることを確認する
    // 【テスト内容】: 両条件が揃った場合のみ2件を返すモックを登録し、チップクリック後の
    //   URL・カード件数を検証する
    // 【期待される動作】: チップクリックごとにURL同期・再フェッチが起こり、最終的に2件のみ表示される
    // 🔵 信頼性レベル: requirements.md §0.4（filterStateToQuery配列先頭採用）・testcases.md TC-101-01 準拠

    // 【テストデータ準備】: is_favorite=true かつ tag_id=SF の場合のみ2件返すモックを登録
    await mockItemsRouteByQuery(page, (query) => {
      if (query.get('is_favorite') === 'true' && query.get('tag_id') === 'SF') {
        return mockFilteredItems;
      }
      return null;
    });

    await page.goto('/general-media');

    // 【実際の処理実行】: 「⭐ お気に入り」チップと「#SF」チップを順にクリックする
    const favResP = page.waitForResponse((r) => r.url().includes('/api/v1/items'));
    await page.getByRole('button', { name: /お気に入り/ }).click();
    await favResP;

    const tagResP = page.waitForResponse((r) => r.url().includes('/api/v1/items'));
    await page.getByRole('button', { name: '#SF' }).click();
    await tagResP;

    // 【結果検証】: URLに両条件が反映されていること
    expect(page.url()).toContain('is_favorite=true'); // 【確認内容】: お気に入り条件がURLに反映されている
    expect(page.url()).toContain('tag_id=SF'); // 【確認内容】: タグ条件がURLに反映されている

    // 【期待値確認】: 両条件を満たす2件のみ表示されること
    await expect(page.locator('.media-card')).toHaveCount(2); // 【確認内容】: AND絞り込み結果のカード件数が一致する
  });

  test('TC-101-02: タイトル検索の即時部分一致（送信ボタン不要）で結果が絞り込まれる', async ({ page }) => {
    // 【テスト目的】: fill('星屑') のみで title=星屑 が URL/APIクエリへ反映され、
    //   部分一致カードのみ表示されることを確認する
    // 【テスト内容】: title=星屑 に部分一致する2件を返すモックを登録し、検索入力後の
    //   URL・カード件数・タイトル内容を検証する
    // 【期待される動作】: 入力に応じ即時再フェッチされる（NFR-201: ボタン押下不要）
    // 🔵 信頼性レベル: SearchBox.tsx（デバウンスなし即時反映）・testcases.md TC-101-02 準拠

    // 【テストデータ準備】: title クエリに"星屑"を含む場合のみ2件を返すモック
    await mockItemsRouteByQuery(page, (query) => {
      const title = query.get('title');
      if (title && title.includes('星屑')) {
        return mockSearchItems;
      }
      return null;
    });

    await page.goto('/general-media');

    // 【実際の処理実行】: 検索ボックスに「星屑」を入力する（送信ボタンはクリックしない）
    const resP = page.waitForResponse((r) => r.url().includes('/api/v1/items') && r.url().includes('title'));
    await page.getByRole('textbox', { name: 'タイトルで検索' }).fill('星屑');
    await resP;

    // 【結果検証】: URLにtitleクエリが反映されていること
    expect(page.url()).toContain('title=%E6%98%9F%E5%B1%91'); // 【確認内容】: 検索語がURLエンコードされて反映されている（星屑）

    // 【期待値確認】: 部分一致した2件のみ表示され、各タイトルに「星屑」を含むこと
    await expect(page.locator('.media-card')).toHaveCount(2); // 【確認内容】: 部分一致結果の件数が一致する
    const titles = await page.locator('.media-card .title').allTextContents();
    for (const t of titles) {
      expect(t).toContain('星屑'); // 【確認内容】: 表示された各カードタイトルに検索語が含まれる
    }
  });

  test('TC-103-01: 無限スクロールで下端到達時にpage=2が発火しカードが追記される', async ({ page }) => {
    // 【テスト目的】: sentinel可視化 → fetchNextPage() → page=2リクエスト → 既存カードの下に追記
    //   されることを確認する
    // 【テスト内容】: page=1(20件)/page=2(20件)、total:40 のページ別モックを登録し、
    //   下端スクロール後のカード件数を検証する
    // 【期待される動作】: page=1表示後に最下部スクロールでpage=2を取得し、計40件になる
    // 🔵 信頼性レベル: testcases.md TC-103-01・useInfiniteScrollSentinel実装照合済み

    // 【テストデータ準備】: page=1/page=2で異なる20件ずつを返すモックを登録
    await mockItemsRouteByPage(page, {
      page1Items: mockItemsForScroll.slice(0, 20),
      page2Items: mockItemsForScroll.slice(20, 40),
      total: 40,
    });

    // 【実際の処理実行】: 初期表示（page=1、20件）を待つ
    const page1ResP = page.waitForResponse((r) => r.url().includes('/api/v1/items'));
    await page.goto('/general-media');
    await page1ResP;
    await expect(page.locator('.media-card')).toHaveCount(20);

    // 【実際の処理実行】: 下端までスクロールしてpage=2の取得を待つ（固定待機ではなくwaitForResponse使用）
    const page2ResP = page.waitForResponse(
      (r) => r.url().includes('/api/v1/items') && r.url().includes('page=2'),
    );
    await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
    await page2ResP;

    // 【結果検証】: カードが計40件に増えていること
    await expect(page.locator('.media-card')).toHaveCount(40); // 【確認内容】: page1+page2の合計件数と一致する
  });

  test('TC-301-01: URLクエリ直接アクセスでフィルタが復元されAPIクエリに反映される', async ({ page }) => {
    // 【テスト目的】: page.goto('/general-media?media_type=anime&is_favorite=true') で初期フィルタがURLから
    //   復元され、APIクエリへ反映されることを確認する
    // 【テスト内容】: 初回GETのクエリにmedia_type=anime, is_favorite=trueが含まれ、
    //   お気に入りチップが選択状態(aria-pressed=true)になることを検証する
    // 【期待される動作】: useItemFiltersがuseSearchParamsから初期化される
    // 🟡 信頼性レベル: requirements.md 追加確認 TC-301-01（妥当な推測を含む）

    // 【テストデータ準備】: クエリに関わらず1件を返す単純モック（本テストの主眼はクエリ内容の検証）
    const { getLastQuery } = await mockItemsRouteByQuery(page, () => mockFilteredItems);

    // 【実際の処理実行】: クエリ付きURLへ直接アクセスする
    const resP = page.waitForResponse((r) => r.url().includes('/api/v1/items'));
    await page.goto('/general-media?media_type=anime&is_favorite=true');
    await resP;

    // 【結果検証】: 初回APIクエリにmedia_type=anime, is_favorite=trueが含まれること
    const lastQuery = getLastQuery();
    expect(lastQuery?.get('media_type')).toBe('anime'); // 【確認内容】: URLのmedia_typeがAPIクエリに反映されている
    expect(lastQuery?.get('is_favorite')).toBe('true'); // 【確認内容】: URLのis_favoriteがAPIクエリに反映されている

    // 【期待値確認】: お気に入りチップが選択状態(aria-pressed=true)であること
    // 【補足】: getByRole の名前一致は .media-card の aria-label（例:「SFお気に入り作品1 の詳細を開く」）
    //   にも部分マッチしうるため、filter-bar コンテナ配下に限定して一意化する
    await expect(
      page.locator('[data-testid="filter-bar"]').getByRole('button', { name: '⭐ お気に入り' }),
    ).toHaveAttribute('aria-pressed', 'true'); // 【確認内容】: URL由来のフィルタ状態がUIに反映されている
  });

  // ==============================
  // 2. 異常系テストケース
  // ==============================

  test('TC-001-E01: APIエラー時にエラー状態と再試行導線が表示され、再試行で成功に復帰する', async ({ page }) => {
    // 【テスト目的】: 初回5xx応答時にErrorStateが表示され、「再試行」クリックで成功応答に
    //   復帰することを確認する
    // 【テスト内容】: 1回目を500エラー、2回目を成功(20件)で出し分けるモックを登録し、
    //   エラーメッセージ表示→再試行→カード表示復帰を検証する
    // 【期待される動作】: APIエラー時にクラッシュせずErrorStateを描画し、再試行で復旧できる
    // 🟡 信頼性レベル: testcases.md TC-001-E01・EDGE-001 照合済み（回数出し分けは妥当な推測）

    // 【テストデータ準備】: TanStack Query既定のリトライ(初回+3回=計4回)をすべて失敗させたのち
    //   成功させるモックを登録する。failCountが内部リトライ回数以下だと、ユーザーが「再試行」を
    //   押す前に内部リトライで自動復旧してしまいErrorStateが一切表示されないため、4回に設定する。
    await mockItemsRouteErrorThenSuccess(page, { failCount: 4, successItems: mockItems });

    // 【実際の処理実行】: 一覧画面へ遷移する
    // 【補足】: TanStack Query既定のリトライ(3回・指数バックオフ)によりisError確定まで
    //   数秒かかるため、待機タイムアウトを延長する
    await page.goto('/general-media');

    // 【結果検証】: エラーメッセージと再試行ボタンが表示されること
    await expect(
      page.getByText('一覧の取得に失敗しました。再試行してください。'),
    ).toBeVisible({ timeout: RETRY_EXHAUST_TIMEOUT_MS }); // 【確認内容】: エラー時にユーザーへ再試行を促す文言が表示される
    await expect(page.getByRole('button', { name: '再試行' })).toBeVisible(); // 【確認内容】: 再試行ボタンが可視である

    // 【実際の処理実行】: 「再試行」ボタンをクリックする
    const retryResP = page.waitForResponse((r) => r.url().includes('/api/v1/items'));
    await page.getByRole('button', { name: '再試行' }).click();
    await retryResP;

    // 【結果検証】: カードが表示され、エラーメッセージが消えること
    await expect(page.locator('.media-card')).toHaveCount(20); // 【確認内容】: 再試行後に一覧が正常表示される
    await expect(
      page.getByText('一覧の取得に失敗しました。再試行してください。'),
    ).toHaveCount(0); // 【確認内容】: 復旧後にエラーメッセージが表示されていない
  });

  test('TC-001-E02: 再試行後も失敗した場合エラー状態が維持される', async ({ page }) => {
    // 【テスト目的】: 継続障害時に誤って成功UIへ遷移しないことを確認する
    // 【テスト内容】: 全リクエストを500で返すモックを登録し、再試行後もエラー状態が
    //   保持されることを検証する
    // 【期待される動作】: 再試行後も「再試行」ボタンとエラーメッセージが残る
    // 🟡 信頼性レベル: EDGE-001からの妥当な派生（要件定義書に明記はないが堅牢性補完）

    // 【テストデータ準備】: 常に500エラーを返すモックを登録
    await mockItemsRouteAlwaysError(page);

    // 【実際の処理実行】: 一覧画面へ遷移し、エラー表示を確認後に再試行をクリックする
    // 【補足】: TanStack Query既定のリトライ(3回・指数バックオフ)によりisError確定まで
    //   数秒かかるため、待機タイムアウトを延長する
    await page.goto('/general-media');
    await expect(page.getByRole('button', { name: '再試行' })).toBeVisible({ timeout: RETRY_EXHAUST_TIMEOUT_MS });

    const retryResP = page.waitForResponse((r) => r.url().includes('/api/v1/items'));
    await page.getByRole('button', { name: '再試行' }).click();
    await retryResP;

    // 【結果検証】: 再試行後もエラーメッセージ・再試行ボタンが表示され、カードが0件のままであること
    // 【補足】: 再試行後もTanStack Queryの内部リトライがすべて失敗するまで再度数秒かかるため、
    //   タイムアウトを延長する
    await expect(
      page.getByText('一覧の取得に失敗しました。再試行してください。'),
    ).toBeVisible({ timeout: RETRY_EXHAUST_TIMEOUT_MS }); // 【確認内容】: 継続障害時にエラーメッセージが表示され続ける
    await expect(page.getByRole('button', { name: '再試行' })).toBeVisible(); // 【確認内容】: 再試行ボタンが表示され続ける
    await expect(page.locator('.media-card')).toHaveCount(0); // 【確認内容】: エラー継続時にカードが描画されない
  });

  test('TC-101-E01: 検索結果0件のとき空状態メッセージと追加導線が表示される', async ({ page }) => {
    // 【テスト目的】: 該当データが存在しない状態（異常ではないが特別扱いが必要な状態）で
    //   EmptyStateが正しく表示され、エラー表示と区別されることを確認する
    // 【テスト内容】: 存在しない検索語「nonexistent」を入力し、0件応答時の表示を検証する
    // 【期待される動作】: 「該当するアイテムがありません」+ 追加導線が表示され、エラーは出ない
    // 🟡 信頼性レベル: testcases.md TC-101-E01・EDGE-101 照合済み

    // 【テストデータ準備】: 検索語 nonexistent の場合のみ0件を返すモック（他は初期表示用に20件）
    await mockItemsRouteByQuery(page, (query) => {
      const title = query.get('title');
      if (title === 'nonexistent') {
        return [];
      }
      return mockItems;
    });

    await page.goto('/general-media');
    await expect(page.locator('.media-card')).toHaveCount(20);

    // 【実際の処理実行】: 存在しない検索語を入力する
    const resP = page.waitForResponse(
      (r) => r.url().includes('/api/v1/items') && r.url().includes('title=nonexistent'),
    );
    await page.getByRole('textbox', { name: 'タイトルで検索' }).fill('nonexistent');
    await resP;

    // 【結果検証】: 空状態メッセージと追加導線が表示され、カードが0件であること
    await expect(page.getByText('該当するアイテムがありません')).toBeVisible(); // 【確認内容】: 空状態メッセージが表示される
    // 【補足】: titlebarにも同一hrefのリンクが常設されているため、EmptyState内(role=status配下)に限定する
    await expect(page.getByRole('status').locator('a[href="/search-add"]')).toBeVisible(); // 【確認内容】: 追加導線（+ 作品を追加）が表示される
    await expect(page.locator('.media-card')).toHaveCount(0); // 【確認内容】: 該当カードが描画されない
  });

  // ==============================
  // 3. 境界値テストケース
  // ==============================

  test('TC-001-B01: 初期ロードが0件の場合に空状態が表示される', async ({ page }) => {
    // 【テスト目的】: 件数=0という一覧描画とEmptyState描画の分岐境界で、正しくEmptyStateのみ
    //   表示されることを確認する
    // 【テスト内容】: data:[], total:0 を返すモックで初回アクセス時の表示を検証する
    // 【期待される動作】: 0件で .media-card が描画されず EmptyState のみ表示される
    // 🟡 信頼性レベル: EDGE-101・testcases.md TC-001-B01 照合済み

    // 【テストデータ準備】: 空配列を返すモックを登録
    await mockItemsRoute(page, { items: [], total: 0 });

    // 【実際の処理実行】: 一覧画面へ遷移し応答を待つ
    const resP = page.waitForResponse((r) => r.url().includes('/api/v1/items'));
    await page.goto('/general-media');
    await resP;

    // 【結果検証】: カードが0件、空状態メッセージと追加導線が表示されること
    await expect(page.locator('.media-card')).toHaveCount(0); // 【確認内容】: 空配列で例外を出さずカードが0件描画される
    await expect(page.getByText('該当するアイテムがありません')).toBeVisible(); // 【確認内容】: 空状態メッセージが表示される
    // 【補足】: titlebarにも同一hrefのリンクが常設されているため、EmptyState内(role=status配下)に限定する
    await expect(page.getByRole('status').locator('a[href="/search-add"]')).toBeVisible(); // 【確認内容】: 追加導線が表示される
  });

  test('TC-103-B01: 最終ページ到達後は再スクロールしても追加リクエストが発火しない', async ({ page }) => {
    // 【テスト目的】: 累計取得件数がpagination.totalに到達した境界(hasNextPage=false)で、
    //   再度スクロールしてもpage=3が発火しないことを確認する
    // 【テスト内容】: page=1(20件)/page=2(20件)、total:40のモックでpage=2取得後に
    //   再度最下部スクロールし、呼び出し回数が2のまま据え置かれることを検証する
    // 【期待される動作】: 累計40==total40の一致点でgetNextPageParamがundefinedを返し、
    //   fetchNextPage()が呼ばれない
    // 🟡 信頼性レベル: testcases.md TC-103-B01・EDGE-102・hooks.ts実装照合済み

    // 【テストデータ準備】: page=1/page=2で20件ずつ、total:40のページ別モックを登録
    const { getCallCount } = await mockItemsRouteByPage(page, {
      page1Items: mockItemsForScroll.slice(0, 20),
      page2Items: mockItemsForScroll.slice(20, 40),
      total: 40,
    });

    // 【実際の処理実行】: 初期表示（page=1）を待ち、下端スクロールでpage=2を取得する
    const page1ResP = page.waitForResponse((r) => r.url().includes('/api/v1/items'));
    await page.goto('/general-media');
    await page1ResP;

    const page2ResP = page.waitForResponse(
      (r) => r.url().includes('/api/v1/items') && r.url().includes('page=2'),
    );
    await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
    await page2ResP;
    await expect(page.locator('.media-card')).toHaveCount(40);
    expect(getCallCount()).toBe(2); // 【確認内容】: page=1とpage=2の2回のみリクエストされている

    // 【実際の処理実行】: 累計件数がtotalに到達した状態で再度最下部へスクロールする
    await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
    // ネットワークが発火しないことを確認するため、一定時間待って呼び出し回数を確認する
    await page.waitForTimeout(500);

    // 【結果検証】: 呼び出し回数が2のまま増えていないこと（page=3が発火しない）
    expect(getCallCount()).toBe(2); // 【確認内容】: hasNextPage=false後は追加リクエストが発生しない
    await expect(page.locator('.media-card')).toHaveCount(40); // 【確認内容】: カード件数が40件のまま据え置かれる
  });

  test('TC-101-B02: 検索入力を空文字にクリアするとtitleパラメータが省略される', async ({ page }) => {
    // 【テスト目的】: 検索文字列長0がクエリ付与/省略の境界であり、空入力時はtitleパラメータを
    //   含めず全件取得に戻ることを確認する
    // 【テスト内容】: 「星屑」を入力後に空文字へクリアし、クリア後のURL・APIクエリを検証する
    // 【期待される動作】: クリア後のGET /api/v1/itemsクエリにtitleが含まれず、URLからも消える
    // 🟡 信頼性レベル: requirements.md §0.4「空文字はパラメータ省略」からの妥当な検証

    // 【テストデータ準備】: title有無で件数を出し分けるモック（クエリ内容の検証が主眼）
    const { getLastQuery } = await mockItemsRouteByQuery(page, (query) => {
      const title = query.get('title');
      if (title && title.includes('星屑')) {
        return mockSearchItems;
      }
      return mockItems;
    });

    await page.goto('/general-media');

    // 【実際の処理実行】: 検索ボックスに「星屑」を入力する
    const fillResP = page.waitForResponse(
      (r) => r.url().includes('/api/v1/items') && r.url().includes('title'),
    );
    const searchInput = page.getByRole('textbox', { name: 'タイトルで検索' });
    await searchInput.fill('星屑');
    await fillResP;
    expect(page.url()).toContain('title=');

    // 【実際の処理実行】: 検索ボックスを空文字にクリアする
    const clearResP = page.waitForResponse((r) => r.url().includes('/api/v1/items'));
    await searchInput.fill('');
    await clearResP;

    // 【結果検証】: クリア後のAPIクエリにtitleが含まれないこと
    expect(getLastQuery()?.has('title')).toBe(false); // 【確認内容】: 空文字時はtitleパラメータが省略される
    // 【期待値確認】: URLからもtitle=星屑が消えていること
    expect(page.url()).not.toContain('title=%E6%98%9F%E5%B1%91'); // 【確認内容】: URLクエリからも検索語が消えている
  });
});
