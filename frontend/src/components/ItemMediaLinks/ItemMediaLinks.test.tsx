import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import { ItemMediaLinks } from './index'
import type { ItemFile, ItemLink, ItemTrailer } from '@/features/items/types'

function makeLink(overrides: Partial<ItemLink> = {}): ItemLink {
  return {
    id: 'link-1',
    url: 'https://netflix.com/watch/1',
    label: 'Netflix',
    ...overrides,
  }
}

function makeFile(overrides: Partial<ItemFile> = {}): ItemFile {
  return {
    id: 'file-1',
    path: '/files/book.pdf',
    label: '単行本1巻',
    file_type: 'pdf',
    ...overrides,
  }
}

function makeTrailer(overrides: Partial<ItemTrailer> = {}): ItemTrailer {
  return {
    id: 'trailer-1',
    url: 'https://youtube.com/watch?v=abc',
    label: '公式PV',
    ...overrides,
  }
}

describe('ItemMediaLinks', () => {
  it('renders links with label/sub and opens in a new tab', () => {
    render(<ItemMediaLinks links={[makeLink()]} files={[]} trailers={[]} />)

    expect(screen.getByText('🔗 配信: Netflix')).toBeInTheDocument()
    const anchor = screen.getByRole('link', { name: /Netflix（新しいタブで開く）/ })
    expect(anchor).toHaveAttribute('href', 'https://netflix.com/watch/1')
    expect(anchor).toHaveAttribute('target', '_blank')
    expect(anchor).toHaveAttribute('rel', 'noopener noreferrer')
  })

  it('renders multiple links', () => {
    render(
      <ItemMediaLinks
        links={[makeLink(), makeLink({ id: 'link-2', label: 'Disney+', url: 'https://disneyplus.com/watch/2' })]}
        files={[]}
        trailers={[]}
      />,
    )

    expect(screen.getByText('🔗 配信: Netflix')).toBeInTheDocument()
    expect(screen.getByText('🔗 配信: Disney+')).toBeInTheDocument()
  })

  it('renders file icons per file_type (pdf/image/other)', () => {
    render(
      <ItemMediaLinks
        links={[]}
        files={[
          makeFile({ id: 'f1', file_type: 'pdf', label: 'PDFファイル', path: '/files/book.pdf' }),
          makeFile({ id: 'f2', file_type: 'image', label: '画像ファイル', path: '/files/cover.png' }),
          makeFile({ id: 'f3', file_type: 'other', label: 'その他ファイル', path: '/files/misc.bin' }),
        ]}
        trailers={[]}
      />,
    )

    expect(
      screen.getByText((_, element) => element?.textContent === '📄 PDFファイル'),
    ).toBeInTheDocument()
    expect(
      screen.getByText((_, element) => element?.textContent === '🖼️ 画像ファイル'),
    ).toBeInTheDocument()
    expect(
      screen.getByText((_, element) => element?.textContent === '📁 その他ファイル'),
    ).toBeInTheDocument()
    expect(screen.getByText('/files/book.pdf')).toBeInTheDocument()
  })

  it('renders trailers as external links', () => {
    render(<ItemMediaLinks links={[]} files={[]} trailers={[makeTrailer()]} />)

    expect(screen.getByText('▶ トレーラー（公式PV）')).toBeInTheDocument()
    const anchor = screen.getByRole('link', { name: /公式PV（新しいタブで開く）/ })
    expect(anchor).toHaveAttribute('href', 'https://youtube.com/watch?v=abc')
    expect(anchor).toHaveAttribute('target', '_blank')
  })

  it('renders nothing when links/files/trailers are all empty', () => {
    const { container } = render(<ItemMediaLinks links={[]} files={[]} trailers={[]} />)
    expect(container).toBeEmptyDOMElement()
  })

  it('renders only the non-empty group when files is empty', () => {
    render(<ItemMediaLinks links={[makeLink()]} files={[]} trailers={[makeTrailer()]} />)

    expect(screen.getByText('🔗 配信: Netflix')).toBeInTheDocument()
    expect(screen.getByText('▶ トレーラー（公式PV）')).toBeInTheDocument()
    expect(screen.queryByText(/📄|🖼️|📁/)).not.toBeInTheDocument()
  })

  it('falls back to a default label when trailer label is null', () => {
    render(<ItemMediaLinks links={[]} files={[]} trailers={[makeTrailer({ label: null })]} />)
    expect(screen.getByText('▶ トレーラー')).toBeInTheDocument()
  })
})
