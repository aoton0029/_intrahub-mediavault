import { cn } from '@/lib/utils'
import { MediaTypeBadge } from './MediaTypeBadge'
import type { Item } from '@/types'

interface MediaCardProps {
  item: Item
  onClick?: (item: Item) => void
}

/**
 * 【機能概要】: 一覧画面でアイテム1件をカード表示する
 * 【実装方針】: item の各フィールドを対応するDOM要素にマッピングする最小実装
 * 【テスト対応】: TC-MC-N-01〜06, TC-MC-E-01〜02, TC-MC-B-01〜02
 * 🔵 信頼性レベル: requirements.md「2.1 MediaCard」より
 */
export function MediaCard({ item, onClick }: MediaCardProps) {
  // 【クリックハンドラ】: onClick 未指定時も安全に呼び出せるガード
  const handleClick = () => {
    onClick?.(item)
  }

  return (
    <div
      data-testid="media-card"
      onClick={handleClick}
      className={cn(
        'flex cursor-pointer flex-col overflow-hidden rounded-lg border border-border bg-background'
      )}
    >
      {/* 【カバー画像】: coverImageUrl 未設定時もプレースホルダで例外を起こさない */}
      <img
        src={item.coverImageUrl ?? ''}
        alt={item.title}
        className="aspect-[2/3] w-full object-cover"
      />

      <div className="flex flex-col gap-1 p-2">
        <span className="text-sm font-medium">{item.title}</span>

        <MediaTypeBadge mediaType={item.mediaType} />

        {/* 【お気に入り表示】: isFavorite を data-favorite 属性で表現 */}
        <span data-testid="media-card-favorite" data-favorite={String(item.isFavorite)} />

        {/* 【ステータス表示】: status を data-status 属性で表現 */}
        <span data-testid="media-card-status" data-status={item.status} />
      </div>
    </div>
  )
}
