import { useState, useEffect, useCallback, useRef } from 'react'
import { usePrivy } from '@privy-io/react-auth'
import type { Transaction, Stats } from '../types'
import { api } from '../services/api'
import { useToast } from '../context/ToastContext'

const POLL_MS = 5000

export function useTransactions() {
  const { authenticated, getAccessToken } = usePrivy()
  const { toast } = useToast()

  const [transactions, setTransactions] = useState<Transaction[]>([])
  const [stats, setStats]               = useState<Stats | null>(null)
  const [loading, setLoading]           = useState(true)
  const latestIdRef                     = useRef<string | null>(null)

  const fetchAll = useCallback(async () => {
    if (!authenticated) return
    try {
      const token = await getAccessToken()
      const [txs, s] = await Promise.all([
        api.transactions.list(token!),
        api.transactions.stats(token!),
      ])
      setTransactions(txs)
      setStats(s)
      // Seed the latest ID so first poll doesn't false-positive
      if (txs.length > 0 && latestIdRef.current === null) {
        latestIdRef.current = txs[0].id
      }
    } catch (e: any) {
      console.error('[transactions] fetch error:', e.message)
    } finally {
      setLoading(false)
    }
  }, [authenticated, getAccessToken])

  // Initial load
  useEffect(() => { fetchAll() }, [fetchAll])

  // Live poll — checks /latest every 5s, only refetches all if new tx arrived
  useEffect(() => {
    if (!authenticated) return
    const interval = setInterval(async () => {
      try {
        const token = await getAccessToken()
        const { latestId } = await api.transactions.latest(token!)
        if (latestId && latestId !== latestIdRef.current) {
          latestIdRef.current = latestId
          await fetchAll()
          toast('💰 Payment received!', 'success')
        }
      } catch { /* silent — transient network issue */ }
    }, POLL_MS)
    return () => clearInterval(interval)
  }, [authenticated, getAccessToken, fetchAll, toast])

  useEffect(() => {
    if (!authenticated) { setTransactions([]); setStats(null) }
  }, [authenticated])

  return { transactions, stats, loading, refetch: fetchAll }
}