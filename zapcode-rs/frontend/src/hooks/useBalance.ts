import { useState, useEffect } from 'react'
import { sepoliaTokens } from 'starkzap'
import { useMerchant } from '../context/MerchantContext'

const ZUSDC = {
  ...sepoliaTokens.USDC,
  name: 'ZapCode USDC Mock',
  symbol: 'USDC',
  address: (import.meta.env.VITE_ZUSDC_ADDRESS ?? sepoliaTokens.USDC.address) as typeof sepoliaTokens.USDC.address,
}

interface BalanceState {
  raw:     string | null   // formatted total: "USDC 42.50"
  usd:     number | null   // total numeric USDC amount for fiat conversion
  usdc:    number | null   // tracked USDC mock balance
  usdce:   number | null
  loading: boolean
  error:   string | null
}

export function useBalance() {
  const { wallet, merchant } = useMerchant()
  const [state, setState] = useState<BalanceState>({
    raw:     null,
    usd:     null,
    usdc:    null,
    usdce:   null,
    loading: false,
    error:   null,
  })

  useEffect(() => {
    if (!wallet || !merchant) return
    let cancelled = false

    async function fetchBalance() {
      setState(s => ({ ...s, loading: true, error: null }))
      try {
        let usdc  = 0
        let usdce = 0

        try {
          const b = await wallet!.balanceOf(ZUSDC)
          usdc = parseFloat(b.toUnit())
        } catch { /* ignore */ }

        if (cancelled) return

        const total = usdc + usdce

        setState({
          raw:     `USDC ${total.toFixed(2)}`,
          usd:     total,
          usdc,
          usdce,
          loading: false,
          error:   null,
        })
      } catch (e: any) {
        if (cancelled) return
        setState({ raw: null, usd: null, usdc: null, usdce: null, loading: false, error: e.message })
      }
    }

    fetchBalance()
    const refresh = () => { void fetchBalance() }
    window.addEventListener('zapcode:balance-refresh', refresh)
    const interval = setInterval(fetchBalance, 30_000)
    return () => {
      cancelled = true
      window.removeEventListener('zapcode:balance-refresh', refresh)
      clearInterval(interval)
    }
  }, [wallet, merchant])

  return state
}
