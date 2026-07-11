import { expect, test } from "@playwright/test";

test("renders the anime detail layout and section order", async ({ page }) => {
  await page.route("**/items/1", async (route) => {
    await route.fulfill({
      json: {
        success: true,
        data: {
          id: "1",
          media_type: "anime",
          title: "星屑のシンフォニア",
          original_title: "Symphonia of Stardust",
          description: "記憶を失った少女が、星々の欠片を集めながら故郷を目指す群像劇。",
          cover_image_url: null,
          release_date: "2025-04-05",
          homepage_url: "https://stardust-symphonia.example.com",
          status: "in_progress",
          consumed_date: null,
          rating: 4,
          is_favorite: true,
          source: "api",
          external_id: "58214",
          created_at: "2026-07-01T12:00:00",
          updated_at: "2026-07-01T12:00:00",
          detail: {
            episodes: 12,
            status: "Finished Airing",
            season: "2025-spring",
            year: 2025,
            studios: ["Studio Example"],
            source: "Original",
            duration: "24 min per ep",
            trailer_url: null,
            genres: ["Fantasy"],
            rating: 8.5,
            url: null,
            alternative_titles: [],
          },
          tags: [{ id: "tag-1", name: "神作画" }],
          categories: [{ id: "category-1", name: "2026年鑑賞予定" }],
          calibre_links: [],
          streaming_links: [],
        },
      },
    });
  });

  await page.route("**/items/1/groups", async (route) => {
    await route.fulfill({
      json: {
        success: true,
        data: [
          {
            id: "group-1",
            item_id: "1",
            parent_item_id: null,
            group_type: "season",
            group_name: "シーズン1",
            number: 1,
            display_order: 1,
            created_at: "2026-07-01T12:00:00",
            updated_at: "2026-07-01T12:00:00",
          },
        ],
      },
    });
  });

  await page.route("**/groups/group-1/episodes", async (route) => {
    await route.fulfill({
      json: {
        success: true,
        data: [
          {
            id: "episode-1",
            group_id: "group-1",
            episode_number: 1,
            title: "星が墜ちた夜",
            original_title: null,
            air_date: null,
            description: null,
            created_at: "2026-07-01T12:00:00",
            updated_at: "2026-07-01T12:00:00",
          },
        ],
      },
    });
  });

  for (const path of ["staff", "relations", "streaming-links", "mylists", "links", "files", "trailers"]) {
    await page.route(`**/items/1/${path}`, async (route) => {
      const payload = path === "staff"
        ? [{ id: "item-staff-1", item_id: "1", staff_id: "staff-1", role: "監督", character_name: null, staff: { id: "staff-1", external_id: null, name: "新津 明日香", image_url: null, created_at: "2026-07-01T12:00:00" } }]
        : path === "relations"
          ? [{ id: "relation-1", item_id: "1", related_item_id: "2", relation_type: "reference", related_item_title: "星屑のシンフォニア OVA", created_at: "2026-07-01T12:00:00" }]
          : path === "streaming-links"
            ? [{ id: "streaming-1", item_id: "1", platform: "netflix", url: "https://www.netflix.com/title/81234567", created_at: "2026-07-01T12:00:00" }]
            : path === "mylists"
              ? [{ id: "mylist-1", name: "お気に入り原作", created_at: "2026-07-01T12:00:00" }]
              : path === "links"
                ? [{ id: "link-1", item_id: "1", label: "公式サイト", url: "https://stardust-symphonia.example.com", created_at: "2026-07-01T12:00:00" }]
                : path === "files"
                  ? [{ id: "file-1", item_id: "1", path: "/tmp/pamphlet.pdf", label: "パンフレットPDF", file_type: "pdf", calibre_book_id: null, created_at: "2026-07-01T12:00:00" }]
                  : path === "trailers"
                    ? [{ id: "trailer-1", item_id: "1", label: "本予告編", url: "https://video.example.com/trailer1", created_at: "2026-07-01T12:00:00" }]
                    : [];

      await route.fulfill({ json: { success: true, data: payload } });
    });
  }

  await page.goto("/media/1");

  await expect(page.locator(".detail-layout")).toBeVisible();
  await expect(page.locator(".detail-rail")).toBeVisible();
  await expect(page.locator(".detail-main")).toBeVisible();
  await expect(page.locator(".titlebar .breadcrumb")).toContainText("一般メディア / アニメ");

  const headings = await page.locator(".doc-section h3").allTextContents();
  expect(headings).toEqual(["概要", "シーズン構成", "スタッフ", "関連作品", "配信", "リソース"]);

  await expect(page.getByRole("button", { name: "シーズンを追加" })).toBeVisible();
  await expect(page.getByRole("button", { name: "関連作品を追加" })).toBeVisible();
  await expect(page.getByRole("button", { name: "編集" })).toHaveCount(0);
});
