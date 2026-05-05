import { useState, useEffect } from "react";
import { Link, useNavigate } from "react-router-dom";
import { usePrivy } from "@privy-io/react-auth";
import { api } from "../services/api";
import { useMerchant } from "../context/MerchantContext";
import { useToast } from "../context/ToastContext";

const API_URL = import.meta.env.VITE_API_URL as string;

const FEATURES = [
  {
    icon: "⊞",
    title: "Scan & Pay",
    desc: "Print your QR code, place it anywhere. Buyers scan and pay from any Starknet wallet — or log in with email and pay directly from our platform.",
  },
  {
    icon: "◈",
    title: "Live dashboard",
    desc: "See every payment the moment it lands. Sound alert, email notification, full history.",
  },
  {
    icon: "⇌",
    title: "Non-custodial",
    desc: "Zapcode never touches your funds. Your wallet, your keys, always.",
  },
  {
    icon: "◆",
    title: "Zero fees",
    desc: "All buyer transactions are fully sponsored. Pay any amount — gas is always free.",
  },
  {
    icon: "⊕",
    title: "Email login",
    desc: "No MetaMask, no seed phrases at signup. Email login — feels like any web2 app.",
  },
  {
    icon: "→",
    title: "Payment links",
    desc: "Send a payment link via WhatsApp or email. Remote clients pay in seconds.",
  },
];

interface PublicStats {
  merchants: number;
  payments: number;
  uniqueSenders: number;
}

function shortAddress(address: string) {
  if (address.length <= 14) return address;
  return `${address.slice(0, 8)}...${address.slice(-6)}`;
}

export function LandingPage() {
  const { authenticated, login, getAccessToken } = usePrivy();
  const { merchant } = useMerchant();
  const { toast } = useToast();
  const navigate = useNavigate();

  const [stats, setStats] = useState<PublicStats | null>(null);
  const [statsLoading, setStatsLoading] = useState(true);
  const [faucetLoading, setFaucetLoading] = useState(false);

  useEffect(() => {
    fetch(`${API_URL}/api/stats/public`)
      .then((r) => r.json())
      .then((d) => {
        setStats(d);
        setStatsLoading(false);
      })
      .catch(() => setStatsLoading(false));
  }, []);

  async function handleFaucet() {
    if (!authenticated) {
      login();
      return;
    }

    if (!merchant) {
      toast("Finish onboarding before using the faucet", "info");
      navigate("/onboard");
      return;
    }

    setFaucetLoading(true);
    try {
      const token = await getAccessToken();
      if (!token) throw new Error("Please sign in again");
      const result = await api.wallet.faucet(token);
      toast(`10 USDC minted to ${shortAddress(result.address)}.`, "success");
      window.dispatchEvent(new Event("zapcode:balance-refresh"));
      navigate("/dashboard");
    } catch (e: any) {
      toast(e.message ?? "Could not mint USDC", "error");
    } finally {
      setFaucetLoading(false);
    }
  }

  return (
    <div
      className="min-h-screen"
      style={{
        background:
          "radial-gradient(ellipse at 50% -10%, rgba(200,146,42,0.08) 0%, transparent 55%), #080604",
      }}
    >
      {/* Nav */}
      <nav className="flex items-center justify-between px-10 h-[68px] border-b border-white/7">
        <div>
          <span className="font-display text-xl font-semibold text-cream">
            Zapcode
          </span>
          <span className="ml-2 text-[9px] tracking-[0.2em] uppercase text-gold">
            Beta
          </span>
        </div>
        <div className="flex items-center gap-3">
          <span className="hidden sm:block text-[10px] tracking-widest uppercase text-muted">
            Starknet · USDC · Non-custodial
          </span>
          <button
            onClick={() => (authenticated ? navigate("/dashboard") : login())}
            className="cursor-pointer bg-gold text-bg text-xs font-bold tracking-widest uppercase px-5 py-2.5 rounded-xl hover:bg-gold-light transition-all"
          >
            {authenticated ? "Dashboard" : "Get started"}
          </button>
        </div>
      </nav>

      {/* Hero */}
      <div className="text-center px-6 pt-24 pb-16 max-w-3xl mx-auto">
        <div className="inline-flex items-center gap-2 text-[10px] tracking-[0.25em] uppercase text-gold border border-gold/20 bg-gold/5 rounded-full px-4 py-2 mb-8">
          <span className="w-1.5 h-1.5 rounded-full bg-gold" />
          Payments for any business · Anywhere in the world
        </div>
        <h1
          className="font-display font-bold text-cream leading-[0.92] mb-6"
          style={{ fontSize: "clamp(52px, 8vw, 96px)", letterSpacing: "-2px" }}
        >
          Accept USDC.
          <br />
          <em className="italic text-gold">Anywhere.</em>
        </h1>
        <p className="text-[16px] text-muted leading-relaxed max-w-lg mx-auto mb-10">
          Scan-to-pay QR codes for any business. Email login, embedded wallet,
          live payment dashboard. No crypto knowledge required.
        </p>
        <div className="flex items-center justify-center gap-3 flex-wrap">
          <button
            onClick={() => (authenticated ? navigate("/dashboard") : login())}
            className="cursor-pointer bg-gold text-bg font-bold tracking-widest uppercase text-xs px-8 py-4 rounded-xl hover:bg-gold-light hover:-translate-y-px transition-all shadow-[0_8px_32px_rgba(200,146,42,.3)]"
          >
            Create your QR code — free
          </button>
          <Link
            to="/pay/demo"
            className="border border-white/10 text-muted text-xs font-semibold tracking-widest uppercase px-6 py-4 rounded-xl hover:border-white/20 hover:text-cream transition-all"
          >
            See buyer flow →
          </Link>
          <button
            onClick={handleFaucet}
            disabled={faucetLoading}
            className="cursor-pointer border border-gold/40 text-gold font-bold tracking-widest uppercase text-xs px-6 py-4 rounded-xl hover:bg-gold/10 transition-all disabled:opacity-45 disabled:cursor-not-allowed"
          >
            {faucetLoading ? "Minting" : "Faucet"}
          </button>
        </div>
      </div>

      {/* Features grid */}
      <div className="px-10 pb-10 max-w-5xl mx-auto">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {FEATURES.map((f) => (
            <div
              key={f.title}
              className="bg-s1 border border-white/7 rounded-2xl px-6 py-5 hover:border-white/12 transition-colors"
            >
              <span className="text-2xl text-gold block mb-3">{f.icon}</span>
              <h3 className="font-display text-lg font-semibold text-cream mb-1.5">
                {f.title}
              </h3>
              <p className="text-xs text-muted leading-relaxed">{f.desc}</p>
            </div>
          ))}
        </div>
      </div>

      {/* Public stats */}
      <div className="px-10 pb-20 mt-13 max-w-3xl mx-auto">
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
          {[
            {
              label: "Merchants registered",
              value: stats?.merchants,
              icon: "◎",
            },
            {
              label: "Payments received",
              value: stats?.payments,
              icon: "⬡",
            },
            {
              label: "Unique senders",
              value: stats?.uniqueSenders,
              icon: "◑",
            },
          ].map((s) => (
            <div
              key={s.label}
              className="bg-s1 border border-white/7 rounded-2xl px-6 py-5 text-center"
            >
              <span className="text-xl text-gold block mb-2">{s.icon}</span>
              {statsLoading ? (
                <div className="h-8 w-16 bg-white/5 rounded-lg animate-pulse mx-auto mb-2" />
              ) : (
                <p className="font-display text-3xl font-bold text-cream mb-1">
                  {s.value ?? "—"}
                </p>
              )}
              <p className="text-[10px] tracking-widest uppercase text-muted">
                {s.label}
              </p>
            </div>
          ))}
        </div>
      </div>

      <footer className="border-t border-white/7 text-center py-5 text-[10px] text-dim tracking-wide">
        Zapcode · Non-custodial USDC payments · Built on Starknet
      </footer>
    </div>
  );
}
