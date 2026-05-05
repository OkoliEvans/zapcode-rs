import { type ReactNode } from 'react'

export function Label({ children }: { children: ReactNode }) {
  return (
    <span className="text-[10px] tracking-[0.2em] uppercase text-muted font-medium">
      {children}
    </span>
  )
}