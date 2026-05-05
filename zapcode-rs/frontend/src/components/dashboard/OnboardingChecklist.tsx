import type { Merchant } from '../../types'

interface ChecklistProps {
  merchant: Merchant | null
  hasTx:    boolean
}

export function OnboardingChecklist({ merchant, hasTx }: ChecklistProps) {
  const steps = [
    { key: 'account', label: 'Account created',        done: !!merchant },
    { key: 'qr',      label: 'QR code downloaded',     done: !!merchant },
    { key: 'payment', label: 'First payment received',  done: hasTx      },
  ]

  if (steps.every(s => s.done)) return null

  return (
    <div className="bg-s2 border border-white/7 rounded-2xl p-6 mb-6">
      <p className="text-[10px] tracking-[0.2em] uppercase text-gold mb-1">Getting started</p>
      <h3 className="font-display text-xl font-semibold text-cream mb-4">Set up your account</h3>
      <div className="space-y-2.5">
        {steps.map(step => (
          <div key={step.key} className="flex items-center gap-3">
            <span
              className={`w-5 h-5 rounded-full flex items-center justify-center text-[10px] font-bold flex-shrink-0 transition-all
                ${step.done ? 'bg-ok text-bg' : 'bg-white/5 border border-white/10 text-muted'}`}
            >
              {step.done ? '✓' : ''}
            </span>
            <span className={`text-sm transition-colors ${step.done ? 'text-muted line-through' : 'text-cream'}`}>
              {step.label}
            </span>
          </div>
        ))}
      </div>
    </div>
  )
}