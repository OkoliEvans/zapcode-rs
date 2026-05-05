import {
  createContext, useContext, useEffect, useState,
  useCallback, useRef, type ReactNode,
} from 'react'
import { usePrivy } from '@privy-io/react-auth'
import { StarkZap, OnboardStrategy, accountPresets } from 'starkzap'
import type { Merchant } from '../types'
import { api } from '../services/api'

const API_URL = (import.meta.env.VITE_API_URL ?? 'http://localhost:3001') as string

interface MerchantContextValue {
  merchant:    Merchant | null
  wallet:      Awaited<ReturnType<StarkZap['onboard']>>['wallet'] | null
  loading:     boolean
  error:       string | null
  refetch:     () => Promise<void>
  setMerchant: (m: Merchant) => void
}

const MerchantContext = createContext<MerchantContextValue | null>(null)

export function MerchantProvider({ children }: { children: ReactNode }) {
  const { authenticated, getAccessToken } = usePrivy()
  const [merchant, setMerchant] = useState<Merchant | null>(null)
  const [wallet, setWallet]     = useState<MerchantContextValue['wallet']>(null)
  const [loading, setLoading]   = useState(false)
  const [error, setError]       = useState<string | null>(null)

  const getTokenRef = useRef(getAccessToken)
  getTokenRef.current = getAccessToken

  const refetch = useCallback(async () => {
    if (!authenticated) return
    setLoading(true)
    setError(null)
    try {
      const token = await getTokenRef.current()
      const data  = await api.merchants.me(token!)
      setMerchant(data)

      try {
        const sdk = new StarkZap({
          network: merchant?.network ?? 'sepolia',
        })

        const onboard = await sdk.onboard({
          strategy:      OnboardStrategy.Privy,
          accountPreset: accountPresets.argentXV050,
          privy: {
            resolve: async () => {
              const freshToken = await getTokenRef.current()
              const res = await fetch(`${API_URL}/api/wallet/starknet`, {
                method:  'POST',
                headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${freshToken}` },
              })
              const text = await res.text()
              const data = text ? JSON.parse(text) : null
              if (!res.ok) throw new Error(data?.error ?? 'Could not fetch merchant wallet')
              const { wallet: w } = data
              if (!w.publicKey) throw new Error('Wallet has no publicKey yet')
              return {
                walletId:  w.id,
                publicKey: w.publicKey,
                serverUrl: `${API_URL}/api/wallet/sign`,
                // Pass walletType so backend routes to correct Privy signing client
                buildBody: ({ walletId, hash }: { walletId: string; hash: string }) => ({
                  walletId,
                  hash,
                  walletType: w.walletType ?? 'merchant',
                }),
              }
            },
          },
          deploy: 'never',
        })
        setWallet(onboard.wallet)
      } catch (walletErr: any) {
        console.warn('[MerchantContext] StarkZap init skipped:', walletErr.message)
      }
    } catch (e: any) {
      if (!e.message?.includes('404') && !e.message?.includes('Not onboarded')) {
        setError(e.message)
      }
    } finally {
      setLoading(false)
    }
  }, [authenticated])

  useEffect(() => { refetch() }, [refetch])
  useEffect(() => {
    if (!authenticated) { setMerchant(null); setWallet(null) }
  }, [authenticated])

  return (
    <MerchantContext.Provider value={{ merchant, wallet, loading, error, refetch, setMerchant }}>
      {children}
    </MerchantContext.Provider>
  )
}

export function useMerchant() {
  const ctx = useContext(MerchantContext)
  if (!ctx) throw new Error('useMerchant must be used within MerchantProvider')
  return ctx
}
