import { useMerchant } from '../context/MerchantContext'
import { useTransactions } from '../hooks/useTransactions'
import { TransactionTable } from '../components/dashboard/TxTable'

const fmt = (n: number) => `$${n.toFixed(2)}`

export function PaymentsPage() {
  const { merchant } = useMerchant()
  const { transactions, stats, loading } = useTransactions()

  return (
    <div className="px-8 py-8">
      <div className="mb-8">
        <h1 className="font-display text-[32px] font-bold text-cream">Payments</h1>
        <p className="text-sm text-muted mt-1">All USDC transfers to your wallet.</p>
      </div>

      {/* Summary strip */}
      {stats && (
        <div className="flex gap-6 mb-6 text-sm">
          {[
            { label: 'Total received',  val: fmt(stats.totalRevenue) },
            { label: 'Total payments',  val: String(stats.orderCount) },
            { label: 'Avg. payment',    val: fmt(stats.avgOrderValue) },
          ].map(({ label, val }) => (
            <div key={label} className="bg-s2 border border-white/7 rounded-xl px-5 py-3.5 flex-1">
              <div className="text-[10px] tracking-widest uppercase text-muted mb-1">{label}</div>
              <div className="font-display text-2xl font-bold text-cream">{val}</div>
            </div>
          ))}
        </div>
      )}

      <div className="bg-s2 border border-white/7 rounded-2xl">
        <div className="px-6 py-5 border-b border-white/7">
          <h2 className="font-display text-xl font-semibold text-cream">All transactions</h2>
        </div>
        <div className="px-6 py-4">
          <TransactionTable
            transactions={transactions}
            loading={loading}
            currency={merchant?.currency}
          />
        </div>
      </div>
    </div>
  )
}