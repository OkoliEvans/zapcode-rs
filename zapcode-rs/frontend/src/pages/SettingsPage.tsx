import { useState } from "react";
import { usePrivy } from "@privy-io/react-auth";
import { useMerchant } from "../context/MerchantContext";
import { useToast } from "../context/ToastContext";
import { Button, CopyButton, Label, Spinner } from "../components/ui";
import { useBalance } from "../hooks/useBalance";
import { api } from "../services/api";

const API_URL = (import.meta.env.VITE_API_URL ?? "http://localhost:3001") as string;

const CURRENCIES = [
  "USD", "KES", "NGN", "GHS", "UGX", "TZS", "ZAR", "EUR", "GBP", "RWF", "ETB", "XOF",
  "XAF", "EGP", "MAD", "ARS", "BRL", "INR", "PHP", "VND", "IDR", "MYR", "THB",
  "SGD", "AED", "SAR", "TRY", "PLN", "SEK", "NOK", "DKK", "CHF",
];

const USDC_TOKEN_ADDRESS =
  (import.meta.env.VITE_ZUSDC_ADDRESS as string | undefined) ??
  "0x053b40a647cedfca6ca84f542a0fe36736031905a9639a7f19a3c1e66bfd5080";

// Per-country offramp options
function getOfframpOptions(country: string) {
  const binanceBase = 'https://p2p.binance.com/trade/sell/USDT'
  const yellowCard  = { name: 'Yellow Card',  desc: 'Sell USDC for local currency — bank & mobile money', url: 'https://yellowcard.io' }
  const moonpay     = { name: 'MoonPay',      desc: 'USDC → fiat off-ramp, 100+ countries',              url: 'https://moonpay.com' }
  const transak     = { name: 'Transak',      desc: 'USDC → fiat, wide country coverage',                url: 'https://transak.com' }

  const map: Record<string, Array<{ name: string; desc: string; url: string }>> = {
    KE: [{ name: 'Binance P2P', desc: 'Sell USDT for KES via M-Pesa', url: `${binanceBase}?fiat=KES&payment=all-payments` }, yellowCard],
    NG: [{ name: 'Binance P2P', desc: 'Sell USDT for NGN via bank transfer', url: `${binanceBase}?fiat=NGN&payment=all-payments` }, { name: 'Resolva', desc: 'USDC → NGN — Nigeria focused', url: 'https://useresolva.io' }, yellowCard],
    GH: [{ name: 'Binance P2P', desc: 'Sell USDT for GHS via mobile money', url: `${binanceBase}?fiat=GHS&payment=all-payments` }, yellowCard],
    RW: [{ name: 'Binance P2P', desc: 'Sell USDT for RWF via mobile money', url: `${binanceBase}?fiat=RWF&payment=all-payments` }, yellowCard],
    UG: [{ name: 'Binance P2P', desc: 'Sell USDT for UGX via mobile money', url: `${binanceBase}?fiat=UGX&payment=all-payments` }, yellowCard],
    TZ: [{ name: 'Binance P2P', desc: 'Sell USDT for TZS via mobile money', url: `${binanceBase}?fiat=TZS&payment=all-payments` }, yellowCard],
    ZA: [{ name: 'Binance P2P', desc: 'Sell USDT for ZAR via bank transfer', url: `${binanceBase}?fiat=ZAR&payment=all-payments` }, moonpay],
    EG: [{ name: 'Binance P2P', desc: 'Sell USDT for EGP', url: `${binanceBase}?fiat=EGP&payment=all-payments` }, moonpay],
    IN: [{ name: 'Binance P2P', desc: 'Sell USDT for INR via bank transfer', url: `${binanceBase}?fiat=INR&payment=all-payments` }, transak],
    BR: [{ name: 'Binance P2P', desc: 'Sell USDT for BRL via PIX', url: `${binanceBase}?fiat=BRL&payment=all-payments` }, transak],
    AR: [{ name: 'Binance P2P', desc: 'Sell USDT for ARS', url: `${binanceBase}?fiat=ARS&payment=all-payments` }, moonpay],
    PH: [{ name: 'Binance P2P', desc: 'Sell USDT for PHP via GCash', url: `${binanceBase}?fiat=PHP&payment=all-payments` }, transak],
  }

  return map[country?.toUpperCase()] ?? [moonpay, yellowCard, transak]
}

const HOW_TO_STEPS = [
  { step: '1', title: 'Receive USDC', desc: 'Payments land directly in your Zapcode wallet on Starknet. You can see them instantly on your dashboard.' },
  { step: '2', title: 'Send to exchange', desc: 'Use the "Send tokens" section above to transfer your USDC to your Binance, Yellow Card, or MoonPay account address.' },
  { step: '3', title: 'Withdraw locally', desc: 'On the exchange, sell USDC for your local currency and withdraw to your bank account or mobile money.' },
]

export function SettingsPage() {
  const { getAccessToken } = usePrivy();
  const { merchant, wallet, setMerchant } = useMerchant();
  const { toast } = useToast();

  const { raw: balRaw, usd: balUsd } = useBalance();

  const [businessName,  setBusinessName]  = useState(merchant?.businessName ?? "");
  const [currency,      setCurrency]      = useState(merchant?.currency ?? "USD");
  const [logoUrl,       setLogoUrl]       = useState(merchant?.logoUrl ?? "");
  const [logoUploading, setLogoUploading] = useState(false);
  const [saving,        setSaving]        = useState(false);
  const [howToOpen,     setHowToOpen]     = useState(false);

  // Transfer state
  const [toAddress,  setToAddress]  = useState("");
  const [sendAmount, setSendAmount] = useState("");
  const [sendToken]                 = useState<"USDC">("USDC");
  const [sending,    setSending]    = useState(false);
  const [sendTxHash, setSendTxHash] = useState<string | null>(null);
  const [sendErr,    setSendErr]    = useState<string | null>(null);

  const save = async () => {
    setSaving(true);
    try {
      const token   = await getAccessToken();
      const updated = await api.merchants.update(token!, {
        businessName,
        currency,
        logoUrl: logoUrl || undefined,
      });
      setMerchant(updated);
      toast("Settings saved", "success");
    } catch (e: any) {
      toast(e.message, "error");
    } finally {
      setSaving(false);
    }
  };

  const send = async () => {
    if (!merchant || !wallet || !toAddress || !sendAmount) return;
    const parsed = parseFloat(sendAmount);
    if (!parsed || parsed <= 0) return;
    setSending(true);
    setSendErr(null);
    setSendTxHash(null);
    try {
      const tokenAddr = USDC_TOKEN_ADDRESS;
      const res = await fetch(`${API_URL}/api/wallet/paymaster/execute`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          walletId: merchant.walletId,
          address: merchant.walletAddress,
          publicKey: merchant.publicKey,
          calls: [{
          contractAddress: tokenAddr,
          entrypoint:      "transfer",
          calldata:        [toAddress, String(Math.round(parsed * 1_000_000)), "0"],
          }],
        }),
      });
      const text = await res.text();
      const data = text ? JSON.parse(text) : null;
      if (!res.ok) throw new Error(data?.error ?? `HTTP ${res.status}`);
      setSendTxHash(data.transactionHash ?? data.hash);
      setSendAmount("");
      setToAddress("");
      window.dispatchEvent(new Event("zapcode:balance-refresh"));
      toast(`${parsed.toFixed(2)} ${sendToken} sent`, "success");
    } catch (e: any) {
      setSendErr(e.message);
    } finally {
      setSending(false);
    }
  };

  if (!merchant) return null;

  const offrampOptions = getOfframpOptions(merchant.country);

  return (
    <div className="px-8 py-8 max-w-xl">
      <div className="mb-8">
        <h1 className="font-display text-[32px] font-bold text-cream">Settings</h1>
        <p className="text-sm text-muted mt-1">Manage your business profile.</p>
      </div>

      {/* Business profile */}
      <div className="bg-s2 border border-white/7 rounded-2xl p-6 mb-5">
        <h2 className="font-display text-lg font-semibold text-cream mb-5">Business profile</h2>
        <div className="space-y-4">

          <div>
            <Label>Business name</Label>
            <input
              type="text"
              value={businessName}
              onChange={e => setBusinessName(e.target.value)}
              className="mt-1.5 w-full bg-s1 border border-white/10 rounded-xl px-4 py-3 text-sm text-cream placeholder-muted focus:outline-none focus:border-gold transition-colors"
              placeholder="Your business name"
            />
          </div>

          <div>
            <Label>Display currency</Label>
            <p className="text-[11px] text-dim mt-0.5 mb-1.5">
              Your USDC balance will be shown in this currency for reference.
            </p>
            <select
              value={currency}
              onChange={e => setCurrency(e.target.value)}
              className="w-full bg-s1 border border-white/10 rounded-xl px-4 py-3 text-sm text-cream focus:outline-none focus:border-gold transition-colors"
            >
              {CURRENCIES.map(c => <option key={c} value={c}>{c}</option>)}
            </select>
          </div>

          <div>
            <Label>QR Code Logo <span className="text-dim font-normal">(optional)</span></Label>
            <p className="text-[11px] text-dim mt-0.5 mb-2">
              Upload or paste a URL — appears in the center of your QR code.
            </p>

            <label className="flex items-center justify-center gap-2 w-full bg-s1 border border-dashed border-white/20 hover:border-gold/50 rounded-xl px-4 py-3 cursor-pointer transition-colors mb-2">
              <input
                type="file"
                accept="image/*"
                className="hidden"
                onChange={async e => {
                  const file = e.target.files?.[0];
                  if (!file) return;
                  setLogoUploading(true);
                  try {
                    const reader = new FileReader();
                    reader.onload = async ev => {
                      try {
                        const base64 = (ev.target?.result as string).split(",")[1];
                        const token  = await getAccessToken();
                        const { url } = await api.upload.logo(token!, base64);
                        setLogoUrl(url);
                      } catch (err: any) {
                        toast(err.message, "error");
                      } finally {
                        setLogoUploading(false);
                      }
                    };
                    reader.readAsDataURL(file);
                  } catch (err: any) {
                    toast(err.message, "error");
                    setLogoUploading(false);
                  }
                }}
              />
              {logoUploading
                ? <><Spinner size={14} /><span className="text-xs text-muted">Uploading…</span></>
                : <><span className="text-gold text-sm">↑</span><span className="text-xs text-muted">Click to upload image</span></>
              }
            </label>

            <input
              type="url"
              value={logoUrl}
              onChange={e => setLogoUrl(e.target.value)}
              className="w-full bg-s1 border border-white/10 rounded-xl px-4 py-3 text-sm text-cream placeholder-muted focus:outline-none focus:border-gold transition-colors"
              placeholder="Or paste image URL…"
            />

            {logoUrl && (
              <div className="mt-2 flex items-center gap-3">
                <img
                  src={logoUrl}
                  alt="Logo preview"
                  className="w-10 h-10 rounded-lg object-cover border border-white/10"
                  onError={e => (e.currentTarget.style.display = "none")}
                />
                <div className="flex-1 min-w-0">
                  <p className="text-[11px] text-muted truncate">{logoUrl}</p>
                </div>
                <button
                  onClick={() => setLogoUrl("")}
                  className="text-[11px] text-danger hover:opacity-80 cursor-pointer"
                >
                  Remove
                </button>
              </div>
            )}
          </div>

        </div>

        <div className="mt-6">
          <Button onClick={save} loading={saving}>Save changes</Button>
        </div>
      </div>

      {/* Wallet info */}
      <div className="bg-s2 border border-white/7 rounded-2xl p-6 mb-5">
        <h2 className="font-display text-lg font-semibold text-cream mb-4">Wallet</h2>
        <div className="flex items-center justify-between gap-3 bg-s1 border border-white/7 rounded-xl px-4 py-3 mb-3">
          <div className="min-w-0">
            <Label>Starknet address</Label>
            <p className="text-[11px] font-mono text-muted truncate mt-0.5">{merchant.walletAddress}</p>
          </div>
          <CopyButton value={merchant.walletAddress} />
        </div>
        <p className="text-[11px] text-dim leading-relaxed">
          This wallet is owned entirely by you via Privy. Zapcode has no access to your funds.
        </p>
      </div>

      {/* Send / Transfer */}
      <div className="bg-s2 border border-white/7 rounded-2xl p-6 mb-5">
        <h2 className="font-display text-lg font-semibold text-cream mb-1">Send tokens</h2>
        <p className="text-xs text-muted mb-5">Transfer Sepolia USDC to any Starknet address.</p>

        <div className="space-y-3">
          <div className="flex gap-2">
            <button
              disabled
              className="flex-1 py-2.5 rounded-xl border border-gold bg-gold/10 text-gold text-xs font-semibold"
            >
              USDC
            </button>
          </div>

          <div>
            <Label>Recipient address</Label>
            <input
              type="text"
              value={toAddress}
              onChange={e => setToAddress(e.target.value)}
              placeholder="0x..."
              className="mt-1.5 w-full bg-s1 border border-white/10 rounded-xl px-4 py-3 text-sm font-mono text-cream placeholder-muted focus:outline-none focus:border-gold transition-colors"
            />
          </div>

          <div>
            <Label>Amount</Label>
            <div className="relative mt-1.5">
              <input
                type="number"
                min="0.01"
                step="0.01"
                value={sendAmount}
                onChange={e => setSendAmount(e.target.value)}
                placeholder="0.00"
                className="w-full bg-s1 border border-white/10 rounded-xl px-4 py-3 text-sm text-cream placeholder-muted focus:outline-none focus:border-gold transition-colors pr-16"
              />
              <span className="absolute right-4 top-1/2 -translate-y-1/2 text-xs font-semibold text-gold">
                {sendToken}
              </span>
            </div>
          </div>

          {balRaw && (
            <p className="text-[11px] text-muted">
              Available: <span className="text-cream font-medium">{balRaw}</span>
              {sendToken === "USDC" && balUsd !== null && (
                <button
                  onClick={() => setSendAmount(balUsd.toFixed(6))}
                  className="ml-2 text-gold underline cursor-pointer"
                >
                  Max
                </button>
              )}
            </p>
          )}

          {sendErr && <p className="text-xs text-danger">{sendErr}</p>}

          {sendTxHash && (
            <div className="bg-ok/10 border border-ok/30 rounded-xl px-4 py-3">
              <p className="text-xs text-ok font-semibold mb-1">Transaction sent ✓</p>
              <a
                href={`https://sepolia.voyager.online/tx/${sendTxHash}`}
                target="_blank"
                rel="noreferrer"
                className="text-[11px] text-gold underline font-mono"
              >
                {sendTxHash.slice(0, 20)}…
              </a>
            </div>
          )}

          <Button
            onClick={send}
            loading={sending}
            disabled={!wallet || !toAddress || !sendAmount || parseFloat(sendAmount) <= 0}
            className="w-full"
          >
            {sending ? "Sending…" : `Send ${sendAmount ? parseFloat(sendAmount).toFixed(2) + " " + sendToken : sendToken}`}
          </Button>

          {!wallet && (
            <p className="text-[11px] text-dim text-center">
              Wallet initializing — wait a moment then refresh.
            </p>
          )}
        </div>
      </div>

      {/* Cash out */}
      <div className="bg-s2 border border-white/7 rounded-2xl p-6">
        <h2 className="font-display text-lg font-semibold text-cream mb-1">Cash out (offramp)</h2>
        <p className="text-xs text-muted leading-relaxed mb-4">
          Zapcode doesn't process withdrawals — cash out directly from your wallet using one of these options.
        </p>

        {/* How-to guide — collapsible */}
        <button
          onClick={() => setHowToOpen(o => !o)}
          className="w-full flex items-center justify-between px-4 py-3 mb-3 bg-gold/5 border border-gold/20 rounded-xl text-left hover:border-gold/40 transition-colors"
        >
          <span className="text-[11px] font-semibold text-gold">How to cash out — step by step</span>
          <span className="text-gold text-sm">{howToOpen ? '↑' : '↓'}</span>
        </button>

        {howToOpen && (
          <div className="mb-4 space-y-3">
            {HOW_TO_STEPS.map(s => (
              <div key={s.step} className="flex gap-3 px-4 py-3 bg-s1 border border-white/7 rounded-xl">
                <span className="w-6 h-6 rounded-full bg-gold/10 border border-gold/30 text-gold text-[10px] font-bold flex items-center justify-center flex-shrink-0 mt-0.5">
                  {s.step}
                </span>
                <div>
                  <p className="text-xs font-semibold text-cream mb-0.5">{s.title}</p>
                  <p className="text-[11px] text-muted leading-relaxed">{s.desc}</p>
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Dynamic offramp links based on merchant country */}
        {offrampOptions.map(r => (
          <a
            key={r.name}
            href={r.url}
            target="_blank"
            rel="noreferrer"
            className="flex items-center justify-between px-4 py-3 mb-2 bg-s1 border border-white/7 rounded-xl hover:border-gold transition-colors group"
          >
            <div>
              <p className="text-xs font-semibold text-cream">{r.name}</p>
              <p className="text-[11px] text-muted">{r.desc}</p>
            </div>
            <span className="text-muted group-hover:text-gold transition-colors">→</span>
          </a>
        ))}
      </div>
    </div>
  );
}
