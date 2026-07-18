import { expect, test } from "@playwright/test";

test("renders the movie detail layout and section order", async ({ page }) => {
  await page.route("**/items/1", async (route) => {
    await route.fulfill({
      json: {
        success: true,
        data: {
          id: "1",
          media_type: "movie",
          title: "深海のオデッセイ",
          original_title: "The Odyssey of the Deep",
          description: "失われた海底都市を巡る、若き海洋学者たちの冒険譚。圧倒的な映像美と音響設計で高く評価された海洋SFの傑作。",
          cover_image_url: null,
          release_date: "2024-11-01",
          homepage_url: "https://odyssey-of-the-deep.example.com",
          status: "completed",
          consumed_date: null,
          rating: 4,
          is_favorite: true,
          source: "api",
          external_id: "998821",
          created_at: "2026-07-01T12:00:00",
          updated_at: "2026-07-01T12:00:00",
          detail: {
            runtime_minutes: 132,
            original_language: "英語",
            vote_count: 3204,
            collection: "深海シリーズ",
            production_companies: ["Meridian Pictures"],
            genres: ["SF", "冒険"],
            rating: 8.3,
          },
          tags: [{ id: "tag-1", name: "映像美" }],
          categories: [{ id: "category-1", name: "2024年鑑賞" }],
          calibre_links: [],
          streaming_links: [],
        },
      },
    });
  });

  for (const path of ["staff", "relations", "streaming-links", "mylists", "links", "files", "trailers"]) {
    await page.route(`**/items/1/${path}`, async (route) => {
      const payload = path === "staff"
        ? [{ id: "item-staff-1", item_id: "1", staff_id: "staff-1", role: "監督", character_name: null, staff: { id: "staff-1", external_id: null, name: "レイラ・ハーモン", image_url: null, created_at: "2026-07-01T12:00:00" } }]
        : path === "relations"
          ? [{ id: "relation-1", item_id: "1", related_item_id: "2", relation_type: "reference", related_item_title: "深海のオデッセイ2 (続編)", created_at: "2026-07-01T12:00:00" }]
          : path === "streaming-links"
            ? [{ id: "streaming-1", item_id: "1", platform: "disney_plus", url: "https://www.disneyplus.com/movies/xxxxx", created_at: "2026-07-01T12:00:00" }]
            : path === "mylists"
              ? [{ id: "mylist-1", name: "映画館で観た作品", created_at: "2026-07-01T12:00:00" }]
              : path === "links"
                ? [{ id: "link-1", item_id: "1", label: "公式サイト", url: "https://odyssey-of-the-deep.example.com", created_at: "2026-07-01T12:00:00" }]
                : path === "files"
                  ? [{ id: "file-1", item_id: "1", path: "/tmp/pamphlet.png", label: "パンフレット画像", file_type: "image", calibre_book_id: null, created_at: "2026-07-01T12:00:00" }]
                  : path === "trailers"
                    ? [{ id: "trailer-1", item_id: "1", label: "本予告編", url: "https://video.example.com/odyssey-trailer", created_at: "2026-07-01T12:00:00" }]
                    : [];

      await route.fulfill({ json: { success: true, data: payload } });
    });
  }

  await page.goto("/media/1");

  await expect(page.locator(".detail-layout")).toBeVisible();
  await expect(page.locator(".detail-rail")).toBeVisible();
  await expect(page.locator(".detail-main")).toBeVisible();
  await expect(page.locator(".titlebar .breadcrumb")).toContainText("メディア / 映画");

  const headings = await page.locator(".doc-section h3").allTextContents();
  expect(headings).toEqual(["概要", "種別固有情報", "スタッフ", "関連作品", "配信", "リソース"]);

  await expect(page.getByRole("button", { name: "関連作品を追加" })).toBeVisible();
  await expect(page.getByRole("button", { name: "配信サイトを追加" })).toBeVisible();
  await expect(page.getByRole("link", { name: "編集する" })).toBeVisible();
});
