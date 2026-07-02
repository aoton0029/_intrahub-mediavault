import { test, expect } from '@playwright/test'
import {
  mockExternalSearchSuccess,
  mockExternalSearchError,
  mockImportItem,
  mockItemDetail,
} from './helpers/mockApi'

const SEARCH_RESULT = {
  externalId: 'ext-001',
  title: '進撃の巨人',
  coverImageUrl: 'https://example.com/cover.jpg',
  releaseDate: '2013-04-07',
  raw: { id: 'ext-001' },
}

const IMPORTED_ITEM = {
  id: 'item-999',
  title: '進撃の巨人',
  mediaType: 'anime',
  status: 'not_started',
  isFavorite: false,
  source: 'api',
  details: { genreList: [] },
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: '2026-01-01T00:00:00Z',
}

test('TC2: 外部API検索→選択→追加→詳細遷移（正常系）', async ({ page }) => {
  await mockExternalSearchSuccess(page, [SEARCH_RESULT])
  await mockImportItem(page, IMPORTED_ITEM)
  await mockItemDetail(page, IMPORTED_ITEM)

  await page.goto('/search/general')
  await page.getByLabel('検索語').fill('進撃の巨人')
  await page.getByRole('button', { name: '検索' }).click()

  await expect(page.getByText('進撃の巨人').first()).toBeVisible()
  await page.getByRole('button', { name: '追加' }).click()

  await expect(page).toHaveURL(/\/items\/item-999/)
  await expect(page.getByText('追加しました')).toBeVisible()
})

test('TC3: 外部API検索でAPI_KEY_NOT_CONFIGURED（422）発生時の表示', async ({ page }) => {
  await mockExternalSearchError(page, 422, 'API_KEY_NOT_CONFIGURED', 'APIキーが設定されていません')

  await page.goto('/search/general')
  await page.getByLabel('検索語').fill('進撃の巨人')
  await page.getByRole('button', { name: '検索' }).click()

  await expect(page.getByText('APIキーが設定されていません')).toBeVisible({ timeout: 15000 })
  await expect(page.getByRole('link', { name: '手動で追加する' })).toHaveAttribute(
    'href',
    '/items/new/general'
  )
})

test('TC4: 外部API検索でEXTERNAL_API_TIMEOUT（502）発生時の表示', async ({ page }) => {
  await mockExternalSearchError(page, 502, 'EXTERNAL_API_TIMEOUT', '検索がタイムアウトしました')

  await page.goto('/search/general')
  await page.getByLabel('検索語').fill('進撃の巨人')
  await page.getByRole('button', { name: '検索' }).click()

  await expect(page.getByText('検索がタイムアウトしました')).toBeVisible({ timeout: 15000 })
  await expect(page.getByRole('button', { name: '再試行' })).toBeVisible()
})
