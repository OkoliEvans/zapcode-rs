import type { Stats, Merchant } from '../../types'
import { Skeleton } from '../ui'

// ── StatCard ───────────────────────────────────────────────────────────────
interface StatCardProps {
  label:    string
  value:    string
  sub?:     string
  accent?:  string
  loading?: boolean
}

export function StatCard({ label, value, sub, loading = false }: StatCardProps) {
  return (
    <div className="bg-s2 border border-white/7 rounded-2xl px-6 py-5">
      <p className="text-[10px] tracking-[0.2em] uppercase text-muted mb-3">{label}</p>
      {loading
        ? <Skeleton className="h-10 w-32 mb-2" />
        : <p className="font-display text-[38px] font-bold text-cream leading-none">{value}</p>
      }
      {sub && <p className="text-[11px] text-muted mt-2">{sub}</p>}
    </div>
  )
}

// ── OnboardingChecklist ────────────────────────────────────────────────────
interface ChecklistProps {
  merchant:     Merchant | null
  hasTx:        boolean
}

const STEPS = [
  { key: 'account', label: 'Account created',      done: (_m: Merchant | null) => true },
  { key: 'qr',      label: 'QR code downloaded',   done: (m: Merchant | null) => !!m },
  { key: 'payment', label: 'First payment received',done: (_m: Merchant | null, hasTx: boolean) => hasTx },
]

export function OnboardingChecklist({ merchant, hasTx }: ChecklistProps) {
  const allDone = STEPS.every(s => s.done(merchant, hasTx))
  if (allDone) return null

  return (
    <div className="bg-s2 border border-white/7 rounded-2xl p-6 mb-6">
      <p className="text-[10px] tracking-[0.2em] uppercase text-gold mb-1">Getting started</p>
      <h3 className="font-display text-xl font-semibold text-cream mb-4">Set up your account</h3>
      <div className="space-y-2.5">
        {STEPS.map(step => {
          const done = step.done(merchant, hasTx)
          return (
            <div key={step.key} className="flex items-center gap-3">
              <span className={`w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold flex-shrink-0 transition-all
                ${done ? 'bg-ok text-bg' : 'bg-white/5 border border-white/10 text-muted'}`}>
                {done ? '✓' : ''}
              </span>
              <span className={`text-sm transition-colors ${done ? 'text-muted line-through' : 'text-cream'}`}>
                {step.label}
              </span>
            </div>
          )
        })}
      </div>
    </div>
  )
}