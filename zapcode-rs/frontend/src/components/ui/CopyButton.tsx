import { useState } from 'react'

interface CopyButtonProps {
  value: string
  label?: string
}

export function CopyButton({ value, label = 'Copy' }: CopyButtonProps) {
  const [copied, setCopied] = useState(false)

  const copy = () => {
    navigator.clipboard.writeText(value)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <button
      onClick={copy}
      className={[
        'flex items-center gap-1.5 text-[10px] tracking-widest uppercase',
        'px-3 py-1.5 rounded-lg border transition-all',
        copied
          ? 'border-ok text-ok'
          : 'border-white/10 text-muted hover:border-gold hover:text-gold',
      ].join(' ')}
    >
      {copied ? '✓ Copied' : `⎘ ${label}`}
    </button>
  )
}