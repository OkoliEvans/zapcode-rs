import { useMerchant } from '../context/MerchantContext'
import { useTransactions } from '../hooks/useTransactions'
import { useBalance } from '../hooks/useBalance'
import { StatCard } from '../components/dashboard/StatCard'
import { OnboardingChecklist } from '../components/dashboard/OnboardingChecklist'
import { TransactionTable } from '../components/dashboard/TxTable'
import { QRCard } from '../components/dashboard/QRCard'
import { Spinner } from '../components/ui'

const fmt = (n: number) => `$${n.toFixed(2)}`

export function OverviewPage() {
  const { merchant, loading: mLoading }     = useMerchant()
  const { transactions, stats, loading: txLoading } = useTransactions()
  const { raw: balRaw, usd: balUsd, usdc, usdce, loading: balLoading } = useBalance()

  if (mLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <Spinner size={28} />
      </div>
    )
  }

  const currency = merchant?.currency ?? 'USD'
  const fxRate   = merchant?.fx?.rate
  const fxAge    = merchant?.fx?.ageSeconds

  const fxLabel = fxRate
    ? `1 USDC ≈ ${fxRate.toFixed(2)} ${currency}${fxAge !== undefined ? ` · ${fxAge}s ago` : ''}`
    : null

  const balFiat = balUsd !== null && fxRate
    ? `≈ ${(balUsd * fxRate).toFixed(2)} ${currency}`
    : undefined

  const todayFiat = stats && fxRate
    ? `≈ ${(stats.todayRevenue * fxRate).toFixed(2)} ${currency}`
    : undefined

  const totalFiat = stats && fxRate
    ? `≈ ${(stats.totalRevenue * fxRate).toFixed(2)} ${currency}`
    : undefined

  // Show breakdown only when both tokens have balance
  const showBreakdown = !balLoading && usdc !== null && usdce !== null && usdce > 0

  return (
    <div className="px-8 py-8">
      {/* Header */}
      <div className="mb-8">
        <h1 className="font-display text-[32px] font-bold text-cream leading-tight">
          {merchant ? `Good day, ${merchant.businessName}` : 'Dashboard'}
        </h1>
        <div className="flex items-center gap-3 mt-1">
          <span className="flex items-center gap-1.5 text-[10px] text-muted">
            <span className="w-1.5 h-1.5 rounded-full bg-ok shadow-[0_0_6px_#50c878]" />
            Watching for payments
          </span>
          {fxLabel && (
            <>
              <span className="text-white/10">·</span>
              <span className="text-[10px] text-muted">{fxLabel}</span>
            </>
          )}
        </div>
      </div>

      {/* Onboarding checklist */}
      <OnboardingChecklist merchant={merchant} hasTx={transactions.length > 0} />

      {/* Balance banner */}
      <div className="bg-gradient-to-r from-gold/10 to-transparent border border-gold/20 rounded-2xl px-6 py-5 mb-6 flex items-center justify-between">
        <div>
          <p className="text-[10px] tracking-[0.2em] uppercase text-gold mb-1">Wallet balance</p>
          {balLoading ? (
            <div className="h-9 w-36 bg-white/5 rounded-lg animate-pulse" />
          ) : (
            <p className="font-display text-[38px] font-bold text-cream leading-none">
              {balRaw ?? '—'}
            </p>
          )}
          {balFiat && !balLoading && (
            <p className="text-[11px] text-muted mt-1">{balFiat}</p>
          )}
          {/* USDC breakdown */}
          {showBreakdown && (
            <div className="flex items-center gap-3 mt-2">
              <span className="text-[10px] text-dim">
                USDC <span className="text-muted">{usdc!.toFixed(4)}</span>
              </span>
            </div>
          )}
        </div>
        <div className="text-right">
          <p className="text-[10px] text-dim">Live on-chain balance</p>
          <p className="text-[10px] text-dim">Refreshes every 30s</p>
        </div>
      </div>

      {/* Stats grid */}
      <div className="grid grid-cols-2 xl:grid-cols-4 gap-4 mb-8">
        <StatCard
          label="Today's revenue"
          value={stats ? fmt(stats.todayRevenue) : '—'}
          sub={todayFiat ?? `${stats?.todayOrders ?? 0} payment${stats?.todayOrders !== 1 ? 's' : ''}`}
          loading={txLoading}
        />
        <StatCard
          label="This week"
          value={stats ? fmt(stats.weekRevenue) : '—'}
          sub={`${stats?.weekOrders ?? 0} payments`}
          loading={txLoading}
        />
        <StatCard
          label="All-time revenue"
          value={stats ? fmt(stats.totalRevenue) : '—'}
          sub={totalFiat}
          loading={txLoading}
        />
        <StatCard
          label="Avg. payment"
          value={stats ? fmt(stats.avgOrderValue) : '—'}
          sub={`${stats?.orderCount ?? 0} total payments`}
          loading={txLoading}
        />
      </div>

      {/* Main content: txs + QR */}
      <div className="flex gap-6">
        <div className="flex-1 min-w-0 bg-s2 border border-white/7 rounded-2xl">
          <div className="px-6 py-5 border-b border-white/7 flex items-center justify-between">
            <h2 className="font-display text-xl font-semibold text-cream">Recent payments</h2>
            <span className="text-[11px] text-muted">{transactions.length} total</span>
          </div>
          <div className="px-6 py-4">
            <TransactionTable
              transactions={transactions.slice(0, 10)}
              loading={txLoading}
              currency={currency}
              network={merchant?.network}
            />
          </div>
        </div>

        {merchant && (
          <div className="w-[320px] flex-shrink-0">
            <QRCard merchant={merchant} />
          </div>
        )}
      </div>
    </div>
  )
}
