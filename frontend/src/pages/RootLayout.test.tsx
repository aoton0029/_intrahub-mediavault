import { describe, expect, it } from 'vitest'
import { render } from '@testing-library/react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import RootLayout from './RootLayout'

function renderAt(path: string) {
  return render(
    <MemoryRouter initialEntries={[path]}>
      <Routes>
        <Route path="/" element={<RootLayout />}>
          <Route index element={<div>home</div>} />
          <Route path="items/:id" element={<div>item-detail</div>} />
          <Route path="collections/general" element={<div>list</div>} />
        </Route>
      </Routes>
    </MemoryRouter>
  )
}

describe('RootLayout', () => {
  // ===== 正常系 =====

  it('TC-01: 一覧画面では app-shell クラスのみ付与され has-properties は付与されない', () => {
    // 🔵 一覧画面相当のルートでの2カラム構成確認
    const { container } = renderAt('/')
    const shell = container.querySelector('.app-shell')
    expect(shell).not.toBeNull()
    expect(shell?.className).not.toContain('has-properties')
  })

  it('TC-02: アイテム詳細画面では app-shell has-properties 両方のクラスが付与される', () => {
    // 🔵 アイテム詳細画面での3カラム構成確認
    const { container } = renderAt('/items/42')
    const shell = container.querySelector('.app-shell')
    expect(shell).not.toBeNull()
    expect(shell?.className).toContain('has-properties')
  })

  it('TC-03: アイテム詳細画面では properties 列の空要素が描画される', () => {
    // 🔵 properties列は空のプレースホルダのみ
    const { container } = renderAt('/items/42')
    const properties = container.querySelector('.properties')
    expect(properties).not.toBeNull()
    expect(properties?.textContent).toBe('')
  })

  // ===== 異常系 =====

  it('TC-04: 一覧画面以外（collections配下）でも properties 列が表示されない', () => {
    // 🔵 詳細画面以外ではproperties列が誤って表示されないこと
    const { container } = renderAt('/collections/general')
    expect(container.querySelector('.properties')).toBeNull()
    expect(container.querySelector('.app-shell')?.className).not.toContain('has-properties')
  })
})
