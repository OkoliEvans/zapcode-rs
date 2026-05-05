import { usePrivy } from '@privy-io/react-auth'
import { useMerchant } from '../../context/MerchantContext'

interface HeaderProps {
  title?:    string
  subtitle?: string
}

export function Header({ title, subtitle }: HeaderProps) {
  const { user }     = usePrivy()
  const { merchant } = useMerchant()

  const email  = user?.email?.address ?? (user as any)?.google?.email ?? ''
  const letter = (merchant?.businessName?.[0] ?? email[0] ?? 'Z').toUpperCase()

  return (
    <header className="flex items-center justify-between px-8 py-5 border-b border-white/7 bg-s1/50 backdrop-blur-sm sticky top-0 z-10">
      <div>
        {title && (
          <h1 className="font-display text-2xl font-bold text-cream leading-tight">{title}</h1>
        )}
        {subtitle && (
          <p className="text-xs text-muted mt-0.5">{subtitle}</p>
        )}
      </div>

      {/* Avatar */}
      <div className="w-8 h-8 rounded-full bg-gradient-to-br from-gold to-[#6a3a08] flex items-center justify-center font-display text-sm font-bold text-bg flex-shrink-0">
        {letter}
      </div>
    </header>
  )
}