import { useMerchant } from '../context/MerchantContext'
import { QRCard } from '../components/dashboard/QRCard'
import { Spinner } from '../components/ui'

export function QRPage() {
  const { merchant, loading } = useMerchant()

  if (loading) return (
    <div className="flex items-center justify-center min-h-screen"><Spinner size={24} /></div>
  )

  return (
    <div className="px-8 py-8 max-w-xl">
      <div className="mb-8">
        <h1 className="font-display text-[32px] font-bold text-cream">My QR Code</h1>
        <p className="text-sm text-muted mt-1">
          Download, print, and place anywhere — counter, tables, receipts, or website.
        </p>
      </div>

      {merchant
        ? <QRCard merchant={merchant} />
        : <p className="text-sm text-muted">Complete onboarding first to get your QR code.</p>
      }

      {/* Tips */}
      <div className="mt-6 space-y-3">
        {[
          ['Print at 300dpi', 'The downloaded PNG is 800×800 at 300dpi — sharp on any printer.'],
          ['Works offline', 'Your QR code is just your wallet address. It works even if Zapcode is down.'],
          ['Share the link too', 'Send the payment link via WhatsApp or email for remote payments.'],
        ].map(([title, desc]) => (
          <div key={title} className="flex gap-3 bg-s2 border border-white/7 rounded-xl px-4 py-3">
            <span className="text-gold mt-0.5">◆</span>
            <div>
              <p className="text-xs font-semibold text-cream">{title}</p>
              <p className="text-xs text-muted mt-0.5 leading-relaxed">{desc}</p>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}