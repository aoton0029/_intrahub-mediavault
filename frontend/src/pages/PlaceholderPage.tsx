import { useSetTitlebar } from '@/components/layout/useTitlebar'

export function PlaceholderPage({ title }: { title: string }) {
  useSetTitlebar({ title })
  return <p className="text-text-muted">{title} は今後実装予定です。</p>
}
