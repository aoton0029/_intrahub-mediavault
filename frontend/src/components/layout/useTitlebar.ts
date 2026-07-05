import { useContext, useEffect } from 'react'
import { TitlebarContext, DEFAULT_TITLEBAR_STATE, type TitlebarState } from './titlebar-context'

export function useTitlebarState() {
  const ctx = useContext(TitlebarContext)
  if (!ctx) throw new Error('useTitlebarState must be used within a TitlebarProvider')
  return ctx.state
}

export function useSetTitlebar({ title, breadcrumb, action }: TitlebarState) {
  const ctx = useContext(TitlebarContext)
  if (!ctx) throw new Error('useSetTitlebar must be used within a TitlebarProvider')
  const { setState } = ctx
  useEffect(() => {
    setState({ title, breadcrumb, action })
    return () => setState(DEFAULT_TITLEBAR_STATE)
  }, [setState, title, breadcrumb, action])
}
