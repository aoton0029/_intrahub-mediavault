import { test, expect } from '@playwright/test'
import { mockItemsList } from './helpers/mockApi'

const ALL_ITEMS = [
  { id: 'item-1', title: 'アニメ作品', mediaType: 'anime', status: 'not_started', isFavorite: false, source: 'manual', details: { genreList: [] }, createdAt: '2026-01-01T00:00:00Z', updatedAt: '2026-01-01T00:00:00Z' },
  { id: 'item-2', title: '映画作品', mediaType: 'movie', status: 'not_started', isFavorite: false, source: 'manual', details: { genreList: [] }, createdAt: '2026-01-01T00:00:00Z', updatedAt: '2026-01-01T00:00:00Z' },
]

const ANIME_ONLY = [ALL_ITEMS[0]]

test('TC1: 全体一覧の絞り込みとURL同期', async ({ page }) => {
  await mockItemsList(page, (mediaType) => (mediaType === 'anime' ? ANIME_ONLY : ALL_ITEMS))

  await page.goto('/')

  await expect(page.getByTestId('media-card')).toHaveCount(2)

  await page.getByLabel('メディアタイプ').selectOption('anime')

  await expect(page).toHaveURL(/media_type=anime/)
  await expect(page.getByTestId('media-card')).toHaveCount(1)

  await page.goBack()

  await expect(page).not.toHaveURL(/media_type=anime/)
  await expect(page.getByTestId('media-card')).toHaveCount(2)
})
