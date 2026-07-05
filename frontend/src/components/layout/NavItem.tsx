import { NavLink } from 'react-router-dom'
import type { ReactNode } from 'react'

export interface NavItemProps {
  to: string
  icon?: ReactNode
  label: string
  count?: number
  indent?: boolean
}

export function NavItem({ to, icon, label, count, indent }: NavItemProps) {
  return (
    <NavLink
      to={to}
      className={({ isActive }) =>
        [
          'flex items-center gap-2 rounded-app px-2 py-1.5 text-[13px] text-text-muted whitespace-nowrap',
          'hover:bg-bg-surface-hover hover:text-text-primary',
          indent ? 'pl-[22px]' : '',
          isActive ? 'bg-accent-soft text-accent-strong hover:bg-accent-soft hover:text-accent-strong' : '',
        ].join(' ')
      }
    >
      {icon}
      <span>{label}</span>
      {count !== undefined && (
        <span className="ml-auto font-mono text-[11px] text-text-faint">{count}</span>
      )}
    </NavLink>
  )
}
