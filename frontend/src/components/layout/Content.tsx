import type { ReactNode } from 'react'

export function Content({ children }: { children: ReactNode }) {
  return <div className="px-7 pt-6 pb-12">{children}</div>
}
