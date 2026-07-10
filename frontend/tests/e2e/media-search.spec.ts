import { expect, test } from "@playwright/test";

test("renders the media search layout and imported state", async ({ page }) => {
  await page.route("**/items/search?**", async (route) => {
    await route.fulfill({
      json: {
        success: true,
        data: [
          { id: "1", media_type: "anime", provider: "annict", title: "星屑のシンフォニア", thumbnail_url: null },
          { id: "2", media_type: "anime", provider: "annict", title: "緋色の境界、青の余白", thumbnail_url: null },
          { id: "3", media_type: "anime", provider: "annict", title: "星屑のシンフォニア OVA", thumbnail_url: null },
        ],
      },
    });
  });

  await page.route("**/items/import", async (route) => {
    const body = route.request().postDataJSON() as { external_id?: string };

    if (body.external_id === "2") {
      await route.fulfill({
        status: 409,
        json: {
          code: "ITEM_ALREADY_IMPORTED",
          message: "Already imported",
        },
      });
      return;
    }

    await route.fulfill({
      status: 201,
      json: {
        success: true,
        data: { id: "created-item" },
      },
    });
  });

  await page.goto("/media/search");

  await expect(page.getByRole("heading", { name: "検索して追加" })).toBeVisible();
  await expect(page.getByRole("link", { name: "手動で入力する" })).toBeVisible();
  await expect(page.locator(".filter-bar")).toBeVisible();
  await expect(page.getByRole("combobox", { name: "種別" })).toBeVisible();
  await expect(page.getByRole("textbox", { name: "作品名" })).toBeVisible();

  await page.getByRole("textbox", { name: "作品名" }).fill("星屑のシンフォニア");
  await page.getByRole("button", { name: "検索" }).click();

  await expect(page.locator(".card-grid.is-compact")).toBeVisible();
  await expect(page.locator(".media-card.search-result.is-compact")).toHaveCount(3);
  await expect(page.locator(".media-card.search-result .title")).toContainText(["星屑のシンフォニア", "緋色の境界、青の余白", "星屑のシンフォニア OVA"]);
  await expect(page.locator(".media-card.search-result .badge")).toContainText(["アニメ", "アニメ", "アニメ"]);

  await page.locator(".media-card.search-result").nth(1).getByRole("button", { name: "取り込む" }).click();
  await expect(page.locator(".media-card.search-result").nth(1).getByRole("button", { name: "取り込み済み" })).toBeDisabled();
});

test("renders only the API key empty state when the provider key is missing", async ({ page }) => {
  await page.route("**/items/search?**", async (route) => {
    await route.fulfill({
      status: 422,
      json: {
        code: "API_KEY_NOT_CONFIGURED",
        message: "Missing API key",
      },
    });
  });

  await page.goto("/media/search");

  await page.getByRole("combobox", { name: "種別" }).selectOption("movie");
  await page.getByRole("textbox", { name: "作品名" }).fill("インセプション");
  await page.getByRole("button", { name: "検索" }).click();

  await expect(page.locator(".empty-state")).toBeVisible();
  await expect(page.getByText("APIキーが設定されていません")).toBeVisible();
  await expect(page.getByText("この種別の検索には TMDb のAPIキーが必要です。設定画面から登録してください。")).toBeVisible();
  await expect(page.locator(".card-grid.is-compact")).toHaveCount(0);
});
