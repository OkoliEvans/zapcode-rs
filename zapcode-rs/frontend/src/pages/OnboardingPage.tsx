import { useState, useMemo } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { usePrivy } from '@privy-io/react-auth'
import { getName, getCodes } from 'country-list'
import { useMerchant } from '../context/MerchantContext'
import { useToast } from '../context/ToastContext'
import { Button, Spinner } from '../components/ui'
import { api } from '../services/api'

// Map country code → [currency code, currency name]
const COUNTRY_CURRENCY: Record<string, [string, string]> = {
  KE: ['KES', 'Kenyan Shilling'],
  NG: ['NGN', 'Nigerian Naira'],
  GH: ['GHS', 'Ghanaian Cedi'],
  UG: ['UGX', 'Ugandan Shilling'],
  TZ: ['TZS', 'Tanzanian Shilling'],
  RW: ['RWF', 'Rwandan Franc'],
  ET: ['ETB', 'Ethiopian Birr'],
  ZA: ['ZAR', 'South African Rand'],
  SN: ['XOF', 'West African CFA'],
  CI: ['XOF', 'West African CFA'],
  CM: ['XAF', 'Central African CFA'],
  EG: ['EGP', 'Egyptian Pound'],
  MA: ['MAD', 'Moroccan Dirham'],
  US: ['USD', 'US Dollar'],
  GB: ['GBP', 'British Pound'],
  EU: ['EUR', 'Euro'],
  DE: ['EUR', 'Euro'],
  FR: ['EUR', 'Euro'],
  AR: ['ARS', 'Argentine Peso'],
  BR: ['BRL', 'Brazilian Real'],
  IN: ['INR', 'Indian Rupee'],
  PH: ['PHP', 'Philippine Peso'],
  MX: ['MXN', 'Mexican Peso'],
  CO: ['COP', 'Colombian Peso'],
  VN: ['VND', 'Vietnamese Dong'],
}

const STEPS = ['Business', 'Location', 'Done']

// Build full country list from country-list lib, sorted A-Z
const ALL_COUNTRIES = getCodes()
  .map(code => ({ code: code.toUpperCase(), name: getName(code) ?? code }))
  .sort((a, b) => a.name.localeCompare(b.name))

export function OnboardingPage() {
  const { getAccessToken } = usePrivy()
  const { setMerchant }    = useMerchant()
  const { toast }          = useToast()
  const navigate           = useNavigate()

  const [step,         setStep]         = useState(0)
  const [businessName, setBusinessName] = useState('')
  const [country,      setCountry]      = useState('')
  const [currency,     setCurrency]     = useState('USD')
  const [search,       setSearch]       = useState('')
  const [loading,      setLoading]      = useState(false)

  // Filter countries by search
  const filteredCountries = useMemo(() => {
    if (!search.trim()) return ALL_COUNTRIES
    const q = search.toLowerCase()
    return ALL_COUNTRIES.filter(c =>
      c.name.toLowerCase().includes(q) || c.code.toLowerCase().includes(q)
    )
  }, [search])

  const selectCountry = (code: string) => {
    setCountry(code)
    // Auto-set currency from mapping, fallback to USD
    const [curr] = COUNTRY_CURRENCY[code] ?? ['USD', 'US Dollar']
    setCurrency(curr)
    setSearch('')
  }

  const skip = () => {
    setCountry('')
    setCurrency('USD')
    submit(true)
  }

  const submit = async (skipCountry = false) => {
    if (!businessName.trim()) { toast('Business name is required', 'error'); return }
    setLoading(true)
    try {
      const token    = await getAccessToken()
      const merchant = await api.merchants.onboard(token!, {
        businessName: businessName.trim(),
        currency:     skipCountry ? 'USD' : currency,
        country:      skipCountry ? 'US'  : (country || 'US'),
        network:      'sepolia',
      })
      setMerchant(merchant)
      setStep(2)
    } catch (e: any) {
      toast(e.message, 'error')
    } finally {
      setLoading(false)
    }
  }

  const selectedCountryName = country ? (getName(country.toLowerCase()) ?? country) : null
  const [, currencyName]    = COUNTRY_CURRENCY[country] ?? ['USD', 'US Dollar']

  return (
    <div className="min-h-screen flex items-center justify-center px-4"
      style={{ background: 'radial-gradient(ellipse at 50% 0%, rgba(200,146,42,0.06) 0%, transparent 60%), #080604' }}
    >
      <div className="w-full max-w-md">
        {/* Logo */}
        <div className="text-center mb-10">
          <Link to="/" className="inline-block hover:opacity-80 transition-opacity">
            <div className="font-display text-3xl font-bold text-cream">Zapcode</div>
            <div className="text-[10px] tracking-[0.3em] uppercase text-gold mt-1">Merchant Setup</div>
          </Link>
        </div>

        {/* Back link */}
        <div className="mb-6">
          <Link to="/" className="inline-flex items-center gap-1.5 text-[11px] text-muted hover:text-cream transition-colors">
            ← Back to home
          </Link>
        </div>

        {/* Step bar */}
        <div className="flex items-center justify-center gap-2 mb-8">
          {STEPS.map((lbl, i) => (
            <span key={lbl} className="flex items-center gap-2">
              <span className={`w-6 h-6 rounded-full flex items-center justify-center text-[10px] font-bold transition-all
                ${step > i  ? 'bg-ok text-bg' : step === i ? 'bg-gold/10 border border-gold text-gold' : 'bg-white/5 border border-white/10 text-muted'}`}>
                {step > i ? '✓' : i + 1}
              </span>
              <span className={`text-[11px] ${step === i ? 'text-cream' : 'text-muted'}`}>{lbl}</span>
              {i < STEPS.length - 1 && <span className="w-8 h-px bg-white/10" />}
            </span>
          ))}
        </div>

        <div className="bg-s1 border border-white/7 rounded-2xl p-8">

          {/* Step 0: Business name */}
          {step === 0 && (
            <>
              <h2 className="font-display text-2xl font-bold text-cream mb-1">What's your business?</h2>
              <p className="text-sm text-muted mb-6">This name appears on your QR code and payment receipts.</p>
              <input
                type="text"
                value={businessName}
                onChange={e => setBusinessName(e.target.value)}
                onKeyDown={e => e.key === 'Enter' && businessName.trim() && setStep(1)}
                placeholder="e.g. Zawadi Guesthouse"
                autoFocus
                className="w-full bg-s2 border border-white/10 rounded-xl px-4 py-3.5 text-sm text-cream placeholder-muted focus:outline-none focus:border-gold transition-colors mb-6"
              />
              <Button onClick={() => businessName.trim() && setStep(1)} disabled={!businessName.trim()} className="w-full">
                Continue →
              </Button>
            </>
          )}

          {/* Step 1: Country + currency */}
          {step === 1 && (
            <>
              <h2 className="font-display text-2xl font-bold text-cream mb-1">Where are you based?</h2>
              <p className="text-sm text-muted mb-5">
                We use this to show your balance in local currency and suggest the best cash-out options.
              </p>

              {/* Selected country pill */}
              {selectedCountryName && (
                <div className="flex items-center gap-2 mb-3 p-3 bg-gold/5 border border-gold/20 rounded-xl">
                  <span className="text-sm text-cream font-medium">{selectedCountryName}</span>
                  <span className="text-dim mx-1">·</span>
                  <span className="text-[11px] text-gold">{currency} — {currencyName}</span>
                  <button
                    onClick={() => { setCountry(''); setCurrency('USD') }}
                    className="ml-auto text-[10px] text-dim hover:text-danger transition-colors"
                  >
                    ✕
                  </button>
                </div>
              )}

              {/* Search input */}
              <input
                type="text"
                value={search}
                onChange={e => setSearch(e.target.value)}
                placeholder="Search country…"
                className="w-full bg-s2 border border-white/10 rounded-xl px-4 py-3 text-sm text-cream placeholder-muted focus:outline-none focus:border-gold transition-colors mb-2"
              />

              {/* Country list */}
              <div className="max-h-48 overflow-y-auto rounded-xl border border-white/7 bg-s2 mb-5">
                {filteredCountries.length === 0 ? (
                  <p className="text-[11px] text-muted text-center py-4">No countries found</p>
                ) : (
                  filteredCountries.map(c => (
                    <button
                      key={c.code}
                      onClick={() => selectCountry(c.code)}
                      className={`w-full text-left px-4 py-2.5 text-sm transition-colors flex items-center justify-between
                        ${country === c.code
                          ? 'bg-gold/10 text-gold'
                          : 'text-muted hover:bg-white/5 hover:text-cream'
                        }`}
                    >
                      <span>{c.name}</span>
                      {COUNTRY_CURRENCY[c.code] && (
                        <span className="text-[10px] text-dim ml-2 flex-shrink-0">
                          {COUNTRY_CURRENCY[c.code][0]}
                        </span>
                      )}
                    </button>
                  ))
                )}
              </div>

              {/* Network info */}
              <div className="bg-s2 border border-gold/20 rounded-xl px-4 py-3 flex items-center gap-3 mb-5">
                <span className="w-2 h-2 rounded-full bg-ok flex-shrink-0" />
                <div>
                  <p className="text-[11px] font-semibold text-cream">Starknet Sepolia</p>
                  <p className="text-[10px] text-muted mt-0.5">Test network · Sepolia USDC · Near-zero fees</p>
                </div>
              </div>

              <div className="flex gap-3">
                <Button variant="outline" onClick={() => setStep(0)} className="flex-1">← Back</Button>
                <Button
                  onClick={() => submit(false)}
                  loading={loading}
                  disabled={!country}
                  className="flex-1"
                >
                  Create account →
                </Button>
              </div>

              {/* Skip */}
              <button
                onClick={skip}
                disabled={loading}
                className="w-full mt-3 text-[11px] text-dim hover:text-muted transition-colors text-center disabled:opacity-50"
              >
                Skip — I'll set this later
              </button>
            </>
          )}

          {/* Step 2: Done */}
          {step === 2 && (
            <div className="text-center">
              <div className="w-16 h-16 rounded-full bg-ok/10 border-2 border-ok flex items-center justify-center text-2xl mx-auto mb-5">
                ✓
              </div>
              <h2 className="font-display text-2xl font-bold text-cream mb-2">You're live!</h2>
              <p className="text-sm text-muted mb-6 leading-relaxed">
                Your USDC wallet is ready on Starknet Sepolia. Download your QR code and start accepting test payments immediately.
              </p>
              <Button onClick={() => navigate('/dashboard')} className="w-full">Go to dashboard →</Button>
            </div>
          )}
        </div>

        <p className="text-center text-[11px] text-dim mt-6">
          Zapcode is non-custodial · Starknet Sepolia · USDC
        </p>
      </div>
    </div>
  )
}
