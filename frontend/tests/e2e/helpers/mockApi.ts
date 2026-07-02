import type { Page, Route } from '@playwright/test'

const API_BASE = '**/api/v1'

function jsonBody(body: unknown) {
  return {
    contentType: 'application/json',
    body: JSON.stringify(body),
  }
}

async function fulfillOk(route: Route, data: unknown, extra: Record<string, unknown> = {}) {
  await route.fulfill({
    status: 200,
    ...jsonBody({ success: true, data, ...extra }),
  })
}

async function fulfillError(route: Route, status: number, code: string, message: string) {
  await route.fulfill({
    status,
    ...jsonBody({ success: false, error: { code, message } }),
  })
}

/** GET /items（一覧）: URLのmedia_typeクエリに応じて異なるレスポンスを返せるようにする */
export async function mockItemsList(
  page: Page,
  responder: (mediaType: string | null) => unknown[]
) {
  await page.route(`${API_BASE}/items?*`, async (route) => {
    const url = new URL(route.request().url())
    const items = responder(url.searchParams.get('media_type'))
    await fulfillOk(route, items, { pagination: { page: 1, limit: 20, total: items.length } })
  })
  await page.route(`${API_BASE}/items`, async (route) => {
    if (route.request().method() !== 'GET') return route.fallback()
    const items = responder(null)
    await fulfillOk(route, items, { pagination: { page: 1, limit: 20, total: items.length } })
  })
}

/** GET /items/:id（詳細） */
export async function mockItemDetail(page: Page, item: Record<string, unknown>) {
  await page.route(`${API_BASE}/items/${item.id}`, async (route) => {
    if (route.request().method() !== 'GET') return route.fallback()
    await fulfillOk(route, item)
  })
}

/** POST /items（手動追加） */
export async function mockCreateItem(page: Page, createdItem: Record<string, unknown>) {
  await page.route(`${API_BASE}/items`, async (route) => {
    if (route.request().method() !== 'POST') return route.fallback()
    await fulfillOk(route, createdItem)
  })
}

/** PATCH /items/:id（編集） */
export async function mockUpdateItem(page: Page, id: string, updatedItem: Record<string, unknown>) {
  await page.route(`${API_BASE}/items/${id}`, async (route) => {
    if (route.request().method() !== 'PATCH') return route.fallback()
    await fulfillOk(route, updatedItem)
  })
}

/** GET /items/search（外部API検索） 成功 */
export async function mockExternalSearchSuccess(page: Page, results: unknown[]) {
  await page.route(`${API_BASE}/items/search?*`, async (route) => {
    await fulfillOk(route, results)
  })
}

/** GET /items/search（外部API検索） エラー */
export async function mockExternalSearchError(
  page: Page,
  status: number,
  code: string,
  message: string
) {
  await page.route(`${API_BASE}/items/search?*`, async (route) => {
    await fulfillError(route, status, code, message)
  })
}

/** POST /items/import（外部検索結果からの追加） */
export async function mockImportItem(page: Page, importedItem: Record<string, unknown>) {
  await page.route(`${API_BASE}/items/import`, async (route) => {
    if (route.request().method() !== 'POST') return route.fallback()
    await fulfillOk(route, importedItem)
  })
}

/** POST /import/booklog */
export async function mockImportBooklog(page: Page, summary: unknown) {
  await page.route(`${API_BASE}/import/booklog`, async (route) => {
    if (route.request().method() !== 'POST') return route.fallback()
    await fulfillOk(route, summary)
  })
}
