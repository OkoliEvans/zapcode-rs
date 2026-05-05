import type {
  Merchant,
  Transaction,
  Stats,
  PublicMerchant,
  OnboardingData,
  FxRate,
} from "../types";

const BASE = import.meta.env.VITE_API_URL ?? "http://localhost:3001";

async function readJson<T>(res: Response): Promise<T | null> {
  const text = await res.text();
  if (!text.trim()) return null;
  try {
    return JSON.parse(text) as T;
  } catch {
    return null;
  }
}

async function req<T>(
  path: string,
  opts: RequestInit = {},
  token?: string,
): Promise<T> {
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(token ? { Authorization: `Bearer ${token}` } : {}),
    ...((opts.headers as Record<string, string>) ?? {}),
  };
  const res = await fetch(`${BASE}${path}`, { ...opts, headers });
  const data = await readJson<{ error?: string } & T>(res);
  if (!res.ok) throw new Error(data?.error ?? `HTTP ${res.status}`);
  if (!data) throw new Error(`Empty response from ${path}`);
  return data as T;
}

export const api = {
  wallet: {
    faucet: (token: string) =>
      req<{
        address: string;
        amount: string;
        tokenAddress: string;
        transactionHash: string;
      }>("/api/wallet/faucet", { method: "POST", body: JSON.stringify({}) }, token),
  },
  merchants: {
    me: (token: string) => req<Merchant>("/api/merchants/me", {}, token),
    onboard: (token: string, data: OnboardingData) =>
      req<Merchant>(
        "/api/merchants/onboard",
        { method: "POST", body: JSON.stringify(data) },
        token,
      ),
    update: (token: string, data: Partial<Merchant>) =>
      req<Merchant>(
        "/api/merchants/me",
        { method: "PATCH", body: JSON.stringify(data) },
        token,
      ),
    get: (id: string) => req<PublicMerchant>(`/api/merchants/${id}`),
    qrUrl: (id: string) => `${BASE}/api/merchants/${id}/qr.png`,
  },
  transactions: {
    list: (token: string, limit = 50) =>
      req<Transaction[]>(`/api/transactions?limit=${limit}`, {}, token),
    stats: (token: string) => req<Stats>("/api/transactions/stats", {}, token),
    latest: (token: string) =>
      req<{ latestId: string | null; latestAt: string | null }>(
        "/api/transactions/latest",
        {},
        token,
      ),
  },
  upload: {
    logo: async (token: string, base64: string) => {
      const res = await fetch(`${BASE}/api/wallet/upload-logo`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({ image: base64 }),
      });
      const data = await readJson<{ error?: string; url?: string }>(res);
      if (!res.ok) throw new Error(data?.error ?? "Upload failed");
      if (!data?.url) throw new Error("Upload returned no URL");
      return data as { url: string };
    },
  },
  rates: {
    get: (from: string, to: string) =>
      req<FxRate & { from: string; to: string }>(
        `/api/rates?from=${from}&to=${to}`,
      ),
  },
  stats: {
    public: () =>
      req<{ merchants: number; payments: number; uniqueSenders: number }>(
        "/api/stats/public",
      ),
  },
};
