import { createContext, type ReactNode } from 'react'

export interface TitlebarState {
  title: string
  breadcrumb?: ReactNode
  action?: ReactNode
}

export const DEFAULT_TITLEBAR_STATE: TitlebarState = { title: '' }

export interface TitlebarContextValue {
  state: TitlebarState
  setState: (state: TitlebarState) => void
}

export const TitlebarContext = createContext<TitlebarContextValue | undefined>(undefined)
