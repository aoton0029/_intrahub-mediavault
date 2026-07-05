import type { ReactNode } from 'react'

export interface NavSectionProps {
  label?: string
  children: ReactNode
}

export function NavSection({ label, children }: NavSectionProps) {
  return (
    <div className="mb-[18px]">
      {label && (
        <div className="px-2 py-1 text-[11px] tracking-wide text-text-faint uppercase">
          {label}
        </div>
      )}
      <div className="flex flex-col gap-0.5">{children}</div>
    </div>
  )
}
