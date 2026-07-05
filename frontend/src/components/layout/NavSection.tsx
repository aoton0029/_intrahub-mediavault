import type { ReactNode } from 'react'

export interface NavSectionProps {
  label?: string
  children: ReactNode
  className?: string
}

export function NavSection({ label, children, className }: NavSectionProps) {
  return (
    <div className={['mb-[18px]', className].filter(Boolean).join(' ')}>
      {label && (
        <div className="px-2 py-1 text-[11px] tracking-wide text-text-faint uppercase">
          {label}
        </div>
      )}
      <div className="flex flex-col gap-0.5">{children}</div>
    </div>
  )
}
