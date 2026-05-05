import { NavLink, Link } from 'react-router-dom'
import { usePrivy } from '@privy-io/react-auth'
import { useMerchant } from '../../context/MerchantContext'

const NAV = [
  { to: '/dashboard',          label: 'Overview',   icon: '◈' },
  { to: '/dashboard/payments', label: 'Payments',   icon: '⇄' },
  { to: '/dashboard/qr',       label: 'My QR Code', icon: '⊞' },
  { to: '/dashboard/settings', label: 'Settings',   icon: '⚙' },
]

export function Sidebar() {
  const { user, logout } = usePrivy()
  const { merchant }     = useMerchant()

  const email  = user?.email?.address ?? (user as any)?.google?.email ?? ''
  const letter = (merchant?.businessName?.[0] ?? email[0] ?? 'Z').toUpperCase()

  return (
    <aside className="w-[220px] flex-shrink-0 flex flex-col border-r border-white/7 bg-s1 sticky top-0 h-screen">
      {/* Logo — click to return to homepage */}
      <Link to="/" className="block px-6 py-6 border-b border-white/7 hover:opacity-80 transition-opacity">
        <div className="font-display text-xl font-semibold text-cream">Zapcode</div>
        <div className="text-[10px] tracking-[0.22em] uppercase text-gold mt-0.5">Merchant Portal</div>
      </Link>

      {/* Nav */}
      <nav className="flex-1 px-3 py-4 space-y-0.5">
        {NAV.map(({ to, label, icon }) => (
          <NavLink
            key={to}
            to={to}
            end={to === '/dashboard'}
            className={({ isActive }) =>
              `flex items-center gap-3 px-3 py-2.5 rounded-xl text-xs font-medium transition-all
              ${isActive
                ? 'bg-gold/10 text-gold border border-gold/20'
                : 'text-muted hover:text-cream hover:bg-white/5 border border-transparent'
              }`
            }
          >
            <span className="text-base leading-none w-4 text-center">{icon}</span>
            {label}
          </NavLink>
        ))}
      </nav>

      {/* User pill */}
      <div className="px-3 py-4 border-t border-white/7">
        <div className="flex items-center gap-2.5 px-3 py-2.5 rounded-xl bg-s2 border border-white/7">
          <div className="w-7 h-7 rounded-full bg-gradient-to-br from-gold to-[#6a3a08] flex items-center justify-center font-display text-sm font-bold text-bg flex-shrink-0">
            {letter}
          </div>
          <div className="min-w-0">
            <div className="text-[11px] font-medium text-cream truncate">
              {merchant?.businessName ?? 'Setup pending'}
            </div>
            <div className="text-[10px] text-muted truncate">{email}</div>
          </div>
        </div>
        <button
          onClick={logout}
          className="w-full mt-2 text-[10px] tracking-widest uppercase text-muted hover:text-cream py-1.5 transition-colors"
        >
          Sign out
        </button>
      </div>
    </aside>
  )
}