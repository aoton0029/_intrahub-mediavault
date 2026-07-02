import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { test, expect } from '@playwright/test'
import { mockImportBooklog } from './helpers/mockApi'

const __dirname = path.dirname(fileURLToPath(import.meta.url))

const IMPORT_SUMMARY = {
  successCount: 8,
  failureCount: 2,
  failures: [
    { rowNumber: 3, reason: 'タイトルが空です' },
    { rowNumber: 7, reason: '重複するアイテムです' },
  ],
}

test('TC7: インポート実行→結果表示（成功・失敗混在）', async ({ page }) => {
  await mockImportBooklog(page, IMPORT_SUMMARY)

  await page.goto('/settings')
  await page.getByRole('tab', { name: 'インポート' }).click()

  const filePath = path.join(__dirname, 'fixtures', 'booklog-sample.csv')
  await page.locator('#booklog-file').setInputFiles(filePath)

  const booklogSection = page.locator('section', { hasText: 'ブクログ CSVインポート' })
  await booklogSection.getByRole('button', { name: 'インポート実行' }).click()

  await expect(page.getByText('成功: 8件')).toBeVisible()
  await expect(page.getByText('失敗: 2件')).toBeVisible()
  await expect(page.getByText('行 3: タイトルが空です')).toBeVisible()
  await expect(page.getByText('行 7: 重複するアイテムです')).toBeVisible()
})
