import { type ButtonHTMLAttributes, type ReactNode } from 'react'
import { Spinner } from './Spinner'

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'outline' | 'ghost' | 'danger'
  size?:    'sm' | 'md' | 'lg'
  loading?: boolean
  children: ReactNode
}

export function Button({
  variant = 'primary',
  size    = 'md',
  loading = false,
  children,
  className = '',
  disabled,
  ...props
}: ButtonProps) {
  const base = [
    'inline-flex items-center justify-center gap-2 rounded-xl font-semibold',
    'transition-all duration-200 disabled:opacity-35 disabled:cursor-not-allowed tracking-wide',
  ].join(' ')

  const sizes: Record<string, string> = {
    sm: 'text-[11px] px-4 py-2.5 tracking-widest uppercase',
    md: 'text-xs px-5 py-3.5 tracking-widest uppercase',
    lg: 'text-xs px-7 py-4 tracking-widest uppercase',
  }

  const variants: Record<string, string> = {
    primary: 'bg-gold text-bg hover:bg-gold-light hover:-translate-y-px shadow-[0_4px_20px_rgba(200,146,42,.25)]',
    outline: 'border-[1.5px] border-gold text-gold hover:bg-gold/10',
    ghost:   'text-muted hover:text-cream hover:bg-white/5',
    danger:  'bg-danger/10 border border-danger/30 text-danger hover:bg-danger/20',
  }

  return (
    <button
      className={`${base} ${sizes[size]} ${variants[variant]} ${className}`}
      disabled={disabled || loading}
      {...props}
    >
      {loading && <Spinner size={14} />}
      {children}
    </button>
  )
}