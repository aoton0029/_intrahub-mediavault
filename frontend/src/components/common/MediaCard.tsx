import { cn } from '@/lib/utils'
import { MediaTypeBadge } from './MediaTypeBadge'
import type { Item, ItemStatus } from '@/types'

interface MediaCardProps {
  item: Item
  onClick?: (item: Item) => void
}

// 【ステータスマッピング】: ItemStatus（not_started/in_progress/completed）を
// モックアップの3値（none/progress/done）に対応させる
// 🟡 信頼性レベル: 資料に直接記載はないが型定義とモックアップから妥当な推測
const STATUS_DOT_CLASS: Record<ItemStatus, 'none' | 'progress' | 'done'> = {
  not_started: 'none',
  in_progress: 'progress',
  completed: 'done',
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
        'media-card flex cursor-pointer flex-col overflow-hidden rounded-lg border border-border bg-background transition-[border-color,transform] duration-150 hover:-translate-y-0.5 hover:border-[var(--accent)]'
      )}
    >
      {/* 【カバー画像プレースホルダ】: coverImageUrl 未設定時もグラデーションプレースホルダで例外を起こさない */}
      <div
        className="cover relative flex aspect-[2/3] w-full items-end justify-end p-1.5"
        style={{ background: 'linear-gradient(160deg, #33304a, #232323)' }}
      >
        {item.coverImageUrl && (
          <img
            src={item.coverImageUrl}
            alt={item.title}
            className="absolute inset-0 h-full w-full object-cover"
          />
        )}

        {/* 【media-typeバッジのオーバーレイ】: coverコンテナ内に絶対配置してラップする */}
        <div
          className="badge absolute left-1.5 top-1.5 rounded bg-black/55 px-1.5 py-0.5"
        >
          <MediaTypeBadge mediaType={item.mediaType} />
        </div>

        {/* 【お気に入り表示】: isFavorite=true の場合のみ★アイコンを表示 */}
        {item.isFavorite && (
          <span
            className="fav relative text-sm"
            style={{ color: 'var(--favorite)' }}
            data-testid="media-card-favorite"
            data-favorite="true"
          >
            ★
          </span>
        )}
      </div>

      <div className="flex flex-col gap-1 p-2">
        <span className="text-sm font-medium">{item.title}</span>

        {/* 【ステータス表示】: status を .status-dot クラス + data-status 属性で表現 */}
        <span
          className={cn('status-dot', STATUS_DOT_CLASS[item.status])}
          data-testid="media-card-status"
          data-status={item.status}
        />
      </div>
    </div>
  )
}
