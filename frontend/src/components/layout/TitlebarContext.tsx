import { useMemo, useState, type ReactNode } from 'react'
import { TitlebarContext, DEFAULT_TITLEBAR_STATE, type TitlebarState } from './titlebar-context'

export function TitlebarProvider({ children }: { children: ReactNode }) {
  const [state, setState] = useState<TitlebarState>(DEFAULT_TITLEBAR_STATE)
  const value = useMemo(() => ({ state, setState }), [state])
  return <TitlebarContext.Provider value={value}>{children}</TitlebarContext.Provider>
}
