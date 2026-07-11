import { expect, test } from "@playwright/test";

test("renders the settings layout", async ({ page }) => {
  await page.route("**/health", async (route) => {
    await route.fulfill({
      json: {
        success: true,
        data: {
          status: "ok",
        },
      },
    });
  });

  await page.goto("/settings");

  await expect(page.locator(".settings-tabs")).toBeVisible();
  await expect(page.getByRole("button", { name: "API連携" })).toBeVisible();
  await expect(page.getByRole("button", { name: "データインポート" })).toBeVisible();
  await expect(page.getByRole("button", { name: "システム状態" })).toBeVisible();
  await expect(page.locator(".settings-panel .panel-api")).toBeVisible();
  await expect(page.locator(".panel-api .kv-card")).toHaveCount(7);

  await page.getByRole("button", { name: "システム状態" }).click();

  await expect(page.locator(".panel-system .kv-card")).toHaveCount(1);
  await expect(page.locator(".panel-system .tag-pill")).toBeVisible();
});
