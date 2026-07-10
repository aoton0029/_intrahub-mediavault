import { expect, test } from "@playwright/test";

test("renders the academic books list layout", async ({ page }) => {
  await page.route("**/items?**", async (route) => {
    await route.fulfill({
      json: {
        success: true,
        data: [
          { id: "1", media_type: "academic_book", title: "分散システム設計の原理", status: "done", rating: 4.5, is_favorite: true, tags: [], categories: [] },
          { id: "2", media_type: "academic_book", title: "認知言語学の展開", status: "done", rating: 3.0, is_favorite: false, tags: [], categories: [] },
        ],
        pagination: { has_more: false, next_after_created_at: null, next_after_id: null },
      },
    });
  });

  await page.route("**/tags", async (route) => {
    await route.fulfill({ json: { success: true, data: [{ id: "tag-1", name: "積読", item_count: 1 }] } });
  });

  await page.route("**/categories", async (route) => {
    await route.fulfill({ json: { success: true, data: [] } });
  });

  await page.goto("/academic-books?tag_id=tag-1");

  await expect(page.locator(".filter-toolbar")).toBeVisible();
  await expect(page.getByRole("combobox", { name: "種別" })).toHaveCount(0);
  await expect(page.getByRole("combobox", { name: "並び順" })).toBeVisible();
  await expect(page.getByRole("textbox", { name: "タイトル検索" })).toBeVisible();
  await expect(page.locator(".filter-bar .chip")).toContainText(["すべて", "", "積読"]);
  await expect(page.locator(".card-grid.is-compact")).toBeVisible();
  await expect(page.locator(".badge").first()).toHaveText("学術書");
  await expect(page.locator(".rating-stars-mini")).toHaveCount(2);
  await expect(page.getByText("分散システム設計の原理")).toBeVisible();
  await expect(page.getByText("認知言語学の展開")).toBeVisible();
});
