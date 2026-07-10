import { expect, test } from "@playwright/test";

test("renders the general media list layout", async ({ page }) => {
  await page.route("**/items?**", async (route) => {
    await route.fulfill({
      json: {
        success: true,
        data: [
          { id: "1", media_type: "anime", title: "攻殻機動隊", status: "done", rating: 4.5, is_favorite: true, tags: [], categories: [] },
          { id: "2", media_type: "movie", title: "リング", status: "done", rating: null, is_favorite: false, tags: [], categories: [] },
        ],
        pagination: { has_more: false, next_after_created_at: null, next_after_id: null },
      },
    });
  });

  await page.route("**/tags", async (route) => {
    await route.fulfill({ json: { success: true, data: [] } });
  });

  await page.route("**/categories", async (route) => {
    await route.fulfill({ json: { success: true, data: [] } });
  });

  await page.goto("/media");

  await expect(page.locator(".filter-toolbar")).toBeVisible();
  await expect(page.getByRole("combobox", { name: "種別" })).toBeVisible();
  await expect(page.getByRole("textbox", { name: "タイトル検索" })).toBeVisible();
  await expect(page.locator(".card-grid.is-compact")).toBeVisible();
  await expect(page.locator(".badge").first()).toBeVisible();
  await expect(page.getByText("攻殻機動隊")).toBeVisible();
  await expect(page.getByText("リング")).toBeVisible();
});
