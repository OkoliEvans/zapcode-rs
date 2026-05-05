# Zapcode-rs

> USDC payment QR system for merchants on Starknet, rebuilt with a Rust backend and Starkzap-rs.

Merchants sign up with email, get a Starknet wallet, and immediately have a QR code they can print and place anywhere. Buyers pay directly on the Zapcode platform with just an email login — no wallet app, no seed phrase, no gas. Zapcode watches for incoming USDC transfers and notifies the merchant in real time via dashboard and email.

**Zapcode never holds funds.** All wallets are owned by their users via Privy. Zapcode is purely monitoring, notification, and dashboard.

**Zero transaction fees.** All buyer payments are fully gas-sponsored by AVNU paymaster. Neither party ever needs to hold STRK or ETH.

---

## Features

### For Merchants

- **Email signup** — no MetaMask, no seed phrases. Sign up like any web2 app.
- **Instant wallet** — Starknet wallet created automatically on signup, prefunded with STRK for deployment.
- **QR code** — print-ready PNG, downloadable in one tap. Works offline — the QR encodes the wallet address directly.
- **Custom QR logo** — upload a business logo to embed in the center of the QR code via ImgBB.
- **Live dashboard** — see every payment the moment it lands. Balance shown in both USDC and local currency equivalent.
- **Email notifications** — payment received email sent automatically on every confirmed transaction.
- **Faucet** — mint 10 test USDC directly to the connected merchant wallet from the dashboard. Handles account deployment automatically if the wallet is new.
- **Send tokens** — transfer USDC to any Starknet address directly from Settings.
- **Multi-country** — balance displayed in merchant's local currency (KES, NGN, GHS, RWF, and 100+ others).
- **Offramp guide** — step-by-step cash-out instructions with country-specific platform recommendations.
- **Zero fees** — all in-platform transfers are gas-sponsored. Merchants never need to hold STRK.

### For Buyers

- **Pay on platform** — log in with email, get a wallet automatically, pay without leaving the page.
- **Pay with existing wallet** — copy address and pay from Argent, Braavos, or any Starknet wallet.
- **Local currency context** — live exchange rate shown on the pay page so buyers can calculate amounts.
- **Zero fees** — all buyer transactions are fully gas-sponsored by AVNU. Buyers never need STRK or ETH.
- **Transaction receipt** — Voyager explorer link shown after payment completes.

---

## How It Works

### Merchant Flow

1. Sign up with email → Starknet wallet created + STRK prefunded automatically
2. Optionally upload logo → download branded QR code → print and place anywhere
3. Buyer scans QR → payment lands in wallet → dashboard + email notification fires
4. Go to Settings → Send tokens to exchange → cash out to bank or mobile money

### Faucet Flow

1. Merchant clicks **Faucet** on the dashboard
2. Backend loads the connected merchant Privy wallet
3. If undeployed, treasury sends STRK for deployment
4. Starkzap-rs deploys the account with `FeeMode::UserPays` if needed
5. Merchant wallet calls `ZUSDC.mint(...)` via `FeeMode::Paymaster`
6. UI shows `10 USDC minted to 0x...` and refreshes the dashboard balance

### Buyer Payment Flow

1. Scan QR → land on Zapcode pay page
2. Log in with email or Google (Privy)
3. Zapcode creates a Starknet wallet silently, prefunds with STRK, deploys account
4. Enter amount → `/api/wallet/paymaster/execute` routes through Starkzap-rs
5. Deploy with `FeeMode::UserPays` if needed, then execute with AVNU paymaster
6. Payment lands directly in the merchant wallet. Gas fully sponsored.

Both on-platform and external wallet options are always available on the same pay page.

### Cash Out Guide (Merchants)

1. Go to **Settings → Send tokens** → transfer USDC to your exchange deposit address
2. On the exchange, sell USDC for local currency
3. Withdraw to bank or mobile money (M-Pesa, MTN MoMo, Airtel Money, etc.)

Platform recommendations shown per country:
- **Kenya** → Binance P2P (KES/M-Pesa) + Yellow Card
- **Nigeria** → Binance P2P (NGN) + Resolva + Yellow Card
- **Ghana, Rwanda, Uganda, Tanzania** → Binance P2P + Yellow Card
- **South Africa, Egypt** → Binance P2P + MoonPay
- **All other countries** → MoonPay + Yellow Card + Transak

---

## Project Structure

```text
zapcode-rs/
├── Cargo.toml                         # Rust workspace — backend + worker + Starkzap-rs
├── migrations/
│   └── 20240101000000_initial_schema.sql
├── contracts/
│   ├── Scarb.toml
│   └── src/
│       └── usdc_mock.cairo            # Sepolia ZUSDC mock (6 decimals), displayed as USDC in UI
├── src/
│   ├── main.rs                        # Axum API server
│   ├── bin/
│   │   └── worker.rs                  # Starknet event watcher process
│   ├── db/
│   │   ├── mod.rs                     # Diesel async pool
│   │   └── schema.rs                  # Diesel schema
│   ├── models.rs                      # Merchant, Transaction, Buyer models
│   ├── routes/
│   │   ├── merchants.rs               # Onboard, profile, public merchant, QR
│   │   ├── transactions.rs            # History, stats, latest polling
│   │   ├── wallet.rs                  # Privy wallets, faucet, Starkzap paymaster execute
│   │   ├── rates.rs                   # USDC to fiat rates
│   │   └── stats.rs                   # Public landing stats
│   ├── services/
│   │   ├── email.rs                   # SMTP email templates
│   │   ├── privy.rs                   # Privy auth, wallet creation, raw sign
│   │   ├── qr.rs                      # QR PNG generation
│   │   ├── rates.rs                   # FX providers and cache
│   │   ├── starknet.rs                # STRK prefund and token helpers
│   │   └── watcher.rs                 # Transfer event polling
│   └── utils/
│       └── auth.rs                    # Privy JWT auth extractor
└── frontend/
    ├── package.json                   # Vite + React 19
    ├── vite.config.ts
    └── src/
        ├── main.tsx                   # PrivyProvider, MerchantProvider, ToastProvider
        ├── App.tsx                    # Routes and auth guards
        ├── services/api.ts            # Typed fetch wrapper
        ├── context/                   # Merchant and toast contexts
        ├── hooks/                     # Balance and transaction polling
        ├── components/                # Dashboard, layout, and UI components
        └── pages/
            ├── LandingPage.tsx
            ├── OnboardingPage.tsx
            ├── OverviewPage.tsx
            ├── PaymentsPage.tsx
            ├── QRPage.tsx
            ├── SettingsPage.tsx       # Profile, send USDC, offramp guide
            └── PayPage.tsx            # Buyer scan and pay page
```

---

## API Reference

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/health` | No | Server health check |
| GET | `/api/merchants/me` | Yes | Current merchant profile plus FX rate |
| POST | `/api/merchants/onboard` | Yes | Create merchant profile and Privy wallet |
| PATCH | `/api/merchants/me` | Yes | Update business name, currency, country, logo |
| GET | `/api/merchants/:id` | No | Public merchant info for pay page |
| GET | `/api/merchants/:id/qr.png` | No | Print-ready QR PNG |
| GET | `/api/transactions` | Yes | Merchant transaction history |
| GET | `/api/transactions/stats` | Yes | Revenue and order stats |
| GET | `/api/transactions/latest` | Yes | Latest tx marker for dashboard polling |
| POST | `/api/wallet/starknet` | Yes | Create or return buyer Starknet wallet |
| POST | `/api/wallet/sign` | Yes | Privy raw sign relay |
| POST | `/api/wallet/faucet` | Yes | Mint 10 test USDC to connected merchant wallet |
| POST | `/api/wallet/paymaster/execute` | No | Execute sponsored Starkzap-rs calls |
| POST | `/api/wallet/upload-logo` | Yes | Upload merchant logo to ImgBB |
| GET | `/api/rates` | No | FX rate lookup (`?from=USDC&to=KES`) |
| GET | `/api/stats/public` | No | Public landing page counters |

---

## Frontend Routes

| Path | Description |
|---|---|
| `/` | Landing page with public stats |
| `/onboard` | Merchant onboarding |
| `/dashboard` | Overview, balance, stats, recent payments, QR |
| `/dashboard/payments` | Full transaction history |
| `/dashboard/qr` | QR download and sharing |
| `/dashboard/settings` | Profile, send USDC, offramp guide |
| `/pay/:merchantAddress` | Buyer-facing payment page |

---

## Setup

### Prerequisites

- Rust stable
- PostgreSQL
- Node.js and pnpm
- Privy account → [privy.io](https://privy.io)
- Starknet Sepolia RPC URL (Alchemy `v0_10` endpoint recommended)
- AVNU API key → [portal.avnu.fi](https://portal.avnu.fi)
- Treasury Starknet account with STRK for deployment prefunds (standard Argent X, no guardian)
- Starkzap-rs — pulled automatically from [crates.io](https://crates.io/crates/starkzap-rs) via Cargo

### 1. Database

```sh
cd zapcode-rs
psql "$DATABASE_URL" -f migrations/20240101000000_initial_schema.sql
```

### 2. Backend Environment

Create `zapcode-rs/.env`:

```sh
DATABASE_URL=postgresql://user:pass@localhost:5432/zapcode

PRIVY_APP_ID=...
PRIVY_APP_SECRET=...
PRIVY_VERIFICATION_KEY=...

STARKNET_NETWORK=sepolia
STARKNET_RPC_URL=https://starknet-sepolia.g.alchemy.com/starknet/version/rpc/v0_10/YOUR_KEY

TREASURY_ADDRESS=0x...
TREASURY_PRIVATE_KEY=0x...

ZUSDC_ADDRESS=0x...
AVNU_API_KEY=...

GMAIL_USER=your@gmail.com
GMAIL_APP_PASSWORD=xxxx xxxx xxxx xxxx

IMGBB_API_KEY=...

HOST=127.0.0.1
PORT=3001
FRONTEND_URL=http://localhost:5173
POLL_INTERVAL_MS=3000
```

### 3. Frontend Environment

Create `zapcode-rs/frontend/.env`:

```sh
VITE_API_URL=http://localhost:3001
VITE_PRIVY_APP_ID=your_privy_app_id
VITE_ZUSDC_ADDRESS=0x...
```

### 4. Install Frontend Dependencies

```sh
cd zapcode-rs/frontend
pnpm install
```

### 5. Run

```sh
# Terminal 1 — backend API
cd zapcode-rs && cargo run

# Terminal 2 — event watcher
cd zapcode-rs && cargo run --bin worker

# Terminal 3 — frontend
cd zapcode-rs/frontend && pnpm dev --host 127.0.0.1
```

Open `http://localhost:5173`.

---

## Architecture

### Starkzap-rs Powers All Transactions

Built on [Starkzap-rs](https://github.com/OkoliEvans/starkzap-rs) ([crates.io](https://crates.io/crates/starkzap-rs)) — a Rust SDK for seamless Starknet wallet integration.

The core transaction path lives in `src/routes/wallet.rs`:

- Privy signer loaded from stored `wallet_id`, `wallet_address`, and `public_key`
- Starkzap-rs onboards the account with the Argent X preset
- Undeployed accounts are prefunded with STRK by treasury
- Deployment via `FeeMode::UserPays`
- Token execution via `FeeMode::Paymaster(PaymasterConfig::from_env())`

```toml
starkzap-rs = { version = "0.1.0", features = ["full"] }
```

### Zero Fees — Fully Sponsored Transactions

AVNU sponsors all buyer `execute()` calls. The only gas cost in the system is the one-time account deployment per new wallet — covered by treasury STRK prefunds. After deployment, everything is free forever.

### Treasury STRK Prefunding

- New merchants: STRK prefunded at onboarding
- New buyers: STRK prefunded before returning wallet to frontend
- Faucet: treasury prefunds + deploys if needed, then merchant wallet calls `mint()`

Treasury requirements: standard Argent X, no guardian, lowercase hex address.

### Non-Custodial by Design

Zapcode never holds or touches user funds. All wallets are owned by users via Privy. Funds land directly in merchant wallets — Zapcode is purely monitoring, notification, and UX.

### Worker-Based Event Monitoring

The worker polls Starknet `Transfer` events for the configured ZUSDC token, matches recipient addresses to active merchants, inserts transactions, and sends payment emails. RPC cost stays near $0 up to thousands of merchants.

### ZUSDC Displayed as USDC

The backend watches `ZUSDC_ADDRESS` on Sepolia and the frontend labels it `USDC` — matching the intended product language before mainnet deployment.

---

## Tech Stack

| Layer | Tech |
|---|---|
| Backend | Rust, Axum, Tokio |
| Database | PostgreSQL, Diesel, diesel-async |
| Wallet SDK | Starkzap-rs (local crate) |
| Starknet SDK | starknet-rs `0.17` |
| Auth & Wallets | Privy |
| Paymaster | AVNU (all transactions sponsored) |
| Token | Sepolia ZUSDC mock (6 decimals), displayed as USDC |
| Worker | Rust binary polling Starknet Transfer events |
| Email | lettre SMTP |
| QR | `qrcode` + `image` — PNG with optional logo overlay |
| Frontend | Vite, React 19, TypeScript, Tailwind CSS v4 |
| FX Rates | CoinGecko API (60s cache) |
| Package manager | pnpm |