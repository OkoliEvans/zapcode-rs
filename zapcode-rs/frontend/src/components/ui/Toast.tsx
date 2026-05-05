type ToastType = 'success' | 'error' | 'info'

interface ToastProps {
  message: string
  type?:   ToastType
}

export function Toast({ message, type = 'info' }: ToastProps) {
  const icons: Record<ToastType, string> = {
    success: '✓',
    error:   '✕',
    info:    '◆',
  }

  const colors: Record<ToastType, string> = {
    success: 'border-ok/30 bg-ok/10 text-ok',
    error:   'border-danger/30 bg-danger/10 text-danger',
    info:    'border-gold/30 bg-gold/10 text-gold',
  }

  return (
    <div
      className={[
        'flex items-center gap-3 px-4 py-3 rounded-xl border backdrop-blur-sm',
        'shadow-[0_8px_32px_rgba(0,0,0,0.5)] text-sm font-medium',
        colors[type],
      ].join(' ')}
    >
      <span className="text-base leading-none">{icons[type]}</span>
      {message}
    </div>
  )
}