import { cn } from '@/lib/utils'
import { Badge } from '@/components/ui/badge'
import { getMediaTypeAccentClass } from '@/lib/media-type-accent'
import type { MediaType } from '@/types'

interface MediaTypeBadgeProps {
  mediaType: MediaType
}

// 【ラベル変換テーブル】: mediaType → 日本語ラベルの決定的マッピング
// 🟡 信頼性レベル: 設計文書に日本語ラベルの明記なし、妥当な推測
const MEDIA_TYPE_LABEL: Record<MediaType, string> = {
  anime: 'アニメ',
  movie: '映画',
  drama: 'ドラマ',
  manga: '漫画',
  novel: '小説',
  game: 'ゲーム',
  academic_book: '専門書',
  paper: '論文',
}

/**
 * 【機能概要】: MediaType を受け取り、対応するアクセントカラー・日本語ラベルでバッジ表示する
 * 【改善内容】: 独自 span 実装から shadcn/ui の Badge コンポーネントベースに置き換え。Badge の
 *   variant="outline" をベースとし、media_type 固有のアクセントカラーは className で上乗せする
 * 【設計方針】: タスク完了条件「各コンポーネントが shadcn/ui の基底コンポーネントを利用している」を満たすため、
 *   Badge の cva バリアント管理を活用しつつ、getMediaTypeAccentClass() の動的クラスを cn() で合成する
 * 【パフォーマンス】: レンダリングコストは従来の span 実装と同等（追加の状態・副作用なし）
 * 【保守性】: Badge 側の基本スタイル（角丸・パディング・フォント）は shadcn/ui 側に集約され、
 *   本コンポーネントは media_type 固有の関心事（ラベル・アクセント色）のみを担当する
 * 【テスト対応】: TC-MB-N-01, TC-MB-N-02, TC-MB-E-01, TC-MB-B-01
 * 🔵 信頼性レベル: media-type-accent.ts を直接参照。Badge への置換は note.md「アーキテクチャ制約」より
 */
export function MediaTypeBadge({ mediaType }: MediaTypeBadgeProps) {
  // 【型外入力防御】: MediaType の8値に含まれない値が渡されてもクラッシュしないようフォールバック
  const accentClass = getMediaTypeAccentClass(mediaType) ?? ''
  const label = MEDIA_TYPE_LABEL[mediaType] ?? String(mediaType)

  return (
    <Badge
      variant="outline"
      data-testid="media-type-badge"
      className={cn(accentClass)}
    >
      {label}
    </Badge>
  )
}
