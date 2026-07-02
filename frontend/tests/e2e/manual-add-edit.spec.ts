import { test, expect } from '@playwright/test'
import { mockCreateItem, mockUpdateItem, mockItemDetail } from './helpers/mockApi'

const CREATED_ITEM = {
  id: 'item-100',
  title: '手動追加テスト',
  mediaType: 'anime',
  status: 'not_started',
  isFavorite: false,
  source: 'manual',
  details: {},
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: '2026-01-01T00:00:00Z',
}

const UPDATED_ITEM = { ...CREATED_ITEM, title: '手動追加テスト（編集後）' }

test('TC5: 手動追加→保存→詳細遷移（正常系）', async ({ page }) => {
  await mockCreateItem(page, CREATED_ITEM)
  await mockItemDetail(page, CREATED_ITEM)

  await page.goto('/items/new/general')
  await page.getByLabel('タイトル').fill('手動追加テスト')
  await page.getByRole('button', { name: '保存' }).click()

  await expect(page).toHaveURL(/\/items\/item-100/)
  await expect(page.getByText('作成しました')).toBeVisible()
})

test('TC-バリデーション: タイトル未入力で保存すると必須エラーが表示される', async ({ page }) => {
  await page.goto('/items/new/general')
  await page.getByRole('button', { name: '保存' }).click()

  await expect(page.locator('p.text-destructive').first()).toBeVisible()
  await expect(page).toHaveURL(/\/items\/new\/general/)
})

test('TC6: 編集→保存→詳細遷移（正常系）', async ({ page }) => {
  await mockItemDetail(page, CREATED_ITEM)
  await page.goto(`/items/${CREATED_ITEM.id}/edit`)

  await expect(page.getByLabel('タイトル')).toHaveValue(CREATED_ITEM.title)

  await mockUpdateItem(page, CREATED_ITEM.id, UPDATED_ITEM)
  await mockItemDetail(page, UPDATED_ITEM)

  await page.getByLabel('タイトル').fill(UPDATED_ITEM.title)
  await page.getByRole('button', { name: '保存' }).click()

  await expect(page).toHaveURL(new RegExp(`/items/${CREATED_ITEM.id}$`))
  await expect(page.getByText('更新しました')).toBeVisible()
})
