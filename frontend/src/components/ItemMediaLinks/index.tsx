import type { ItemFile, ItemLink, ItemTrailer } from '@/features/items/types'

export interface ItemMediaLinksProps {
  links: ItemLink[]
  files: ItemFile[]
  trailers: ItemTrailer[]
}

const FILE_TYPE_ICON: Record<ItemFile['file_type'], string> = {
  pdf: '📄',
  image: '🖼️',
  other: '📁',
}

/**
 * 詳細画面「リンク・ファイル・トレーラー」セクション（TASK-0017）
 *
 * 02_item_detail.htmlの `.doc-section`/`.prop-list-item`/`.label`/`.sub` 構造に準拠する。
 * links/files/trailersがすべて空の場合はセクション自体を非表示にする。
 */
export function ItemMediaLinks({ links, files, trailers }: ItemMediaLinksProps) {
  if (links.length === 0 && files.length === 0 && trailers.length === 0) {
    return null
  }

  return (
    <div className="doc-section">
      <h3>リンク・ファイル・トレーラー</h3>

      {links.map((link) => (
        <div className="prop-list-item" key={link.id}>
          <span className="label">🔗 配信: {link.label}</span>
          <span className="sub">
            <a
              href={link.url}
              target="_blank"
              rel="noopener noreferrer"
              aria-label={`${link.label}（新しいタブで開く）`}
            >
              {link.url}
            </a>
          </span>
        </div>
      ))}

      {files.map((file) => (
        <div className="prop-list-item" key={file.id}>
          <span className="label">
            <span aria-hidden="true">{FILE_TYPE_ICON[file.file_type]}</span> {file.label ?? file.path}
          </span>
          <span className="sub">{file.path}</span>
        </div>
      ))}

      {trailers.map((trailer) => (
        <div className="prop-list-item" key={trailer.id}>
          <span className="label">▶ トレーラー{trailer.label ? `（${trailer.label}）` : ''}</span>
          <span className="sub">
            <a
              href={trailer.url}
              target="_blank"
              rel="noopener noreferrer"
              aria-label={`${trailer.label ?? 'トレーラー'}（新しいタブで開く）`}
            >
              {trailer.url}
            </a>
          </span>
        </div>
      ))}
    </div>
  )
}
