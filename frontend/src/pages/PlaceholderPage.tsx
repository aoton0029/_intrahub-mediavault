import { AppShell } from '@/components/layout/AppShell'

export function PlaceholderPage({ title }: { title: string }) {
  return (
    <AppShell title={title}>
      <p className="text-text-muted">{title} は今後実装予定です。</p>
    </AppShell>
  )
}
