import { useState } from 'react'
import type { Merchant } from '../../types'
import { Button, CopyButton, Label } from '../ui'
import { api } from '../../services/api'

export function QRCard({ merchant }: { merchant: Merchant }) {
  const [downloading, setDownloading] = useState(false)

  const qrPreviewUrl = `${api.merchants.qrUrl(merchant.walletAddress)}?preview=1`
  const payPageUrl   = `${window.location.origin}/pay/${merchant.walletAddress}`

  const download = async () => {
    setDownloading(true)
    try {
      const res  = await fetch(api.merchants.qrUrl(merchant.walletAddress))
      const blob = await res.blob()
      const url  = URL.createObjectURL(blob)
      const a    = document.createElement('a')
      a.href     = url
      a.download = `${merchant.businessName.replace(/\s+/g, '-')}-zapcode.png`
      a.click()
      URL.revokeObjectURL(url)
    } finally {
      setDownloading(false)
    }
  }

  return (
    <div className="bg-s2 border border-white/7 rounded-2xl p-6">
      <Label>Your QR Code</Label>
      <p className="text-xs text-muted mt-1 mb-5">
        Print it, place it on your counter, tables, or receipts. Works offline — buyers
        can pay even when your internet is down.
      </p>

      {/* QR Preview */}
      <div className="flex justify-center mb-6">
        <div className="bg-[#0d0b08] border border-white/10 rounded-xl p-3 inline-block">
          <img
            src={qrPreviewUrl}
            alt="Your QR code"
            className="w-48 h-48 rounded-lg"
            onError={e => (e.currentTarget.style.display = 'none')}
          />
        </div>
      </div>

      {/* Actions */}
      <div className="space-y-2.5">
        <Button onClick={download} loading={downloading} className="w-full">
          ↓ Download QR PNG (Print-ready)
        </Button>

        <div className="flex items-center gap-2 bg-s1 border border-white/7 rounded-xl px-4 py-3">
          <div className="flex-1 min-w-0">
            <Label>Payment link</Label>
            <p className="text-[11px] font-mono text-muted truncate mt-0.5">{payPageUrl}</p>
          </div>
          <CopyButton value={payPageUrl} label="Copy link" />
        </div>

        <div className="flex items-center gap-2 bg-s1 border border-white/7 rounded-xl px-4 py-3">
          <div className="flex-1 min-w-0">
            <Label>Wallet address</Label>
            <p className="text-[11px] font-mono text-muted truncate mt-0.5">{merchant.walletAddress}</p>
          </div>
          <CopyButton value={merchant.walletAddress} label="Copy" />
        </div>
      </div>

      <p className="text-[11px] text-dim mt-4 leading-relaxed">
        Your QR code works 24/7 regardless of internet connectivity. Zapcode never holds your funds —
        all USDC goes directly to your wallet.
      </p>
    </div>
  )
}