import { useCallback, type KeyboardEvent, type MouseEvent } from 'react'
import { useNavigate } from 'react-router-dom'
import type { Item, ItemStatus, MediaType } from '@/features/items/types'
import { useUpdateItemStatus } from '@/features/items/hooks'
import { StatusDropdown } from '@/components/StatusDropdown'

/**
 * media_type -> バッジ表示文言
 * 🔵 信頼性: 01_home.html .badge 表示（ANIME/MOVIE/MANGA/NOVEL/GAME/DRAMA/ACADEMIC/PAPER）より
 */
const MEDIA_TYPE_BADGE_LABEL: Record<MediaType, string> = {
  anime: 'ANIME',
  movie: 'MOVIE',
  drama: 'DRAMA',
  manga: 'MANGA',
  novel: 'NOVEL',
  game: 'GAME',
  academic_book: 'ACADEMIC',
  paper: 'PAPER',
}

export interface MediaCardProps {
  item: Item
}

export function MediaCard({ item }: MediaCardProps) {
  const navigate = useNavigate()
  const { mutate, isPending } = useUpdateItemStatus()

  const handleActivate = useCallback(() => {
    navigate(`/items/${item.id}`)
  }, [navigate, item.id])

  const handleStatusChange = useCallback(
    (newStatus: ItemStatus) => {
      mutate({ id: item.id, body: { status: newStatus } })
    },
    [mutate, item.id],
  )

  const handleStatusAreaClick = useCallback((event: MouseEvent<HTMLDivElement>) => {
    event.stopPropagation()
  }, [])

  const handleKeyDown = useCallback(
    (event: KeyboardEvent<HTMLDivElement>) => {
      if (event.key === 'Enter' || event.key === ' ') {
        event.preventDefault()
        handleActivate()
      }
    },
    [handleActivate],
  )

  const coverStyle = item.cover_image_url
    ? {
        backgroundImage: `url(${item.cover_image_url})`,
        backgroundSize: 'cover',
        backgroundPosition: 'center',
      }
    : undefined

  return (
    <div
      className="media-card"
      role="button"
      tabIndex={0}
      aria-label={`${item.title} の詳細を開く`}
      onClick={handleActivate}
      onKeyDown={handleKeyDown}
    >
      <div className="cover" style={coverStyle}>
        <span className="badge" aria-hidden="true">
          {MEDIA_TYPE_BADGE_LABEL[item.media_type]}
        </span>
        {item.is_favorite && (
          <span className="fav" aria-hidden="true">
            ★
          </span>
        )}
      </div>
      <div className="body">
        <p className="title">{item.title}</p>
        <div className="meta" onClick={handleStatusAreaClick}>
          <StatusDropdown
            status={item.status}
            onStatusChange={handleStatusChange}
            isPending={isPending}
          />
          {item.season_label && <span> · {item.season_label}</span>}
        </div>
      </div>
    </div>
  )
}
