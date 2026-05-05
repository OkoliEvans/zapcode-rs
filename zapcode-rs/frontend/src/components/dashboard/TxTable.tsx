import type { Transaction } from '../../types'
import { StatusBadge, Skeleton } from '../ui'

interface TxTableProps {
  transactions: Transaction[]
  loading:      boolean
  currency?:    string
  network?:     string
}

function shortAddr(addr: string) {
  return `${addr.slice(0, 10)}…${addr.slice(-6)}`
}

function timeAgo(dateStr: string) {
  const diff = Date.now() - new Date(dateStr).getTime()
  const mins = Math.floor(diff / 60_000)
  if (mins < 1)  return 'just now'
  if (mins < 60) return `${mins}m ago`
  const hrs = Math.floor(mins / 60)
  if (hrs < 24)  return `${hrs}h ago`
  return new Date(dateStr).toLocaleDateString()
}

export function TransactionTable({
  transactions,
  loading,
  currency = 'USD',
  network  = 'sepolia',
}: TxTableProps) {
  const explorerBase = 'https://sepolia.voyager.online/tx'

  if (loading) {
    return (
      <div className="space-y-3">
        {Array.from({ length: 5 }).map((_, i) => (
          <Skeleton key={i} className="h-12 w-full" />
        ))}
      </div>
    )
  }

  if (transactions.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-20 text-center">
        <div className="text-5xl mb-4 opacity-20">⇄</div>
        <p className="text-sm text-muted">No payments yet.</p>
        <p className="text-xs text-dim mt-1">Share your QR code to start accepting USDC.</p>
      </div>
    )
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-xs">
        <thead>
          <tr className="border-b border-white/7">
            {['Time', 'From', 'Amount', 'Status', 'Tx'].map(h => (
              <th
                key={h}
                className="text-left px-4 py-3 text-[10px] tracking-widest uppercase text-muted font-medium first:pl-0"
              >
                {h}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {transactions.map((tx, i) => (
            <tr
              key={tx.id}
              className={`border-b border-white/5 hover:bg-white/[0.02] transition-colors
                ${i === 0 && Date.now() - new Date(tx.detectedAt).getTime() < 10_000
                  ? 'animate-[rise_0.3s_ease]' : ''}`}
            >
              <td className="px-4 py-3.5 text-muted whitespace-nowrap first:pl-0">
                {timeAgo(tx.detectedAt)}
              </td>
              <td className="px-4 py-3.5 font-mono text-muted">
                {shortAddr(tx.fromAddress)}
              </td>
              <td className="px-4 py-3.5 font-semibold text-cream whitespace-nowrap">
                {parseFloat(tx.amount).toFixed(2)}
                <span className="text-gold ml-1 font-normal">USDC</span>
              </td>
              <td className="px-4 py-3.5">
                <StatusBadge status={tx.status} />
              </td>
              <td className="px-4 py-3.5">
                <a
                  href={`${explorerBase}/${tx.txHash}`}
                  target="_blank"
                  rel="noreferrer"
                  className="font-mono text-gold/60 hover:text-gold transition-colors"
                >
                  {tx.txHash.slice(0, 10)}…
                </a>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}
