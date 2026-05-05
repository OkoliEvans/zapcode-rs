# Zapcode-rs

> USDC payment QR system for merchants on Starknet, built with a Rust backend and Starkzap-rs.

Merchants sign up with email, get a Starknet wallet, and immediately have a QR code they can print and place anywhere. Buyers pay directly on the Zapcode platform with just an email login. Zapcode watches for incoming USDC transfers and notifies the merchant in real time via dashboard and email.

Zapcode never holds funds. All wallets are owned by users via Privy.

---

## Features

### For Merchants

- Email signup through Privy
- Starknet wallet created automatically on onboarding
- Print-ready QR code pointing to the merchant pay page
- Optional QR logo upload via ImgBB
- Live dashboard with balance, recent payments, and local currency reference
- Faucet button that mints 10 test USDC to the merchant wallet
- Email notifications for confirmed inbound payments
- Send USDC from Settings to any Starknet address
- Offramp guide with country-specific cash-out suggestions
- Non-custodial: funds land directly in the merchant wallet

### For Buyers

- Public pay page at `/pay/:merchantAddress`
- Email login through Privy with deterministic wallet reuse
- Sponsored payments via AVNU paymaster through Starkzap-rs
- Automatic account readiness: prefund STRK, deploy if needed, execute transfer
- External wallet fallback: pay from Argent, Braavos, or any Starknet wallet

---

## How It Works

### Merchant Flow

1. Merchant signs in with Privy
2. `/api/merchants/onboard` creates a Privy Starknet wallet and stores `wallet_id`, `wallet_address`, and `public_key`
3. Treasury prefunds STRK for the one-time account deployment
4. Merchant downloads or shares the QR code
5. Buyer pays USDC to the merchant wallet
6. Worker watches Starknet transfer events and records new payments
7. Dashboard refreshes and email notification is sent

### Buyer Payment Flow

1. Buyer opens `/pay/:merchantAddress`
2. Buyer logs in with Privy if paying on-platform
3. `/api/wallet/starknet` creates or returns the buyer wallet
4. Backend ensures the buyer account is deployed
5. Frontend builds a USDC transfer call
6. `/api/wallet/paymaster/execute` routes through Starkzap-rs — deploy if needed, then execute with AVNU paymaster
7. Payment lands directly in the merchant wallet

---

## Architecture

Zapcode-rs uses [Starkzap-rs](https://github.com/MistLabs/starkzap-rs) as its core transaction SDK. The key path in `src/routes/wallet.rs`:

- Privy signer loaded from stored `wallet_id`, `wallet_address`, and `public_key`
- Starkzap-rs onboards the account with the Argent X preset
- Undeployed accounts are prefunded with STRK by treasury
- Deployment via `FeeMode::UserPays`
- Token execution via `FeeMode::Paymaster(PaymasterConfig::from_env())`

AVNU sponsors execution calls. New accounts still need STRK for their one-time deployment — treasury handles this. Treasury does not mint faucet tokens.

---

## Project Structure

```text
zapcode-rs/
├── Cargo.toml
├── migrations/
│   └── 20240101000000_initial_schema.sql
├── contracts/
│   ├── Scarb.toml
│   └── src/
│       └── usdc_mock.cairo
├── src/
│   ├── main.rs
│   ├── bin/
│   │   └── worker.rs
│   ├── db/
│   │   ├── mod.rs
│   │   └── schema.rs
│   ├── models.rs
│   ├── routes/
│   │   ├── merchants.rs
│   │   ├── transactions.rs
│   │   ├── wallet.rs
│   │   ├── rates.rs
│   │   └── stats.rs
│   ├── services/
│   │   ├── email.rs
│   │   ├── privy.rs
│   │   ├── qr.rs
│   │   ├── rates.rs
│   │   ├── starknet.rs
│   │   └── watcher.rs
│   └── utils/
│       └── auth.rs
└── frontend/
    ├── package.json
    ├── vite.config.ts
    └── src/
        ├── main.tsx
        ├── App.tsx
        ├── services/api.ts
        ├── context/
        ├── hooks/
        ├── components/
        └── pages/
            ├── LandingPage.tsx
            ├── OnboardingPage.tsx
            ├── OverviewPage.tsx
            ├── PaymentsPage.tsx
            ├── QRPage.tsx
            ├── SettingsPage.tsx
            └── PayPage.tsx
```

---

## API Reference

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/health` | No | Server health check |
| GET | `/api/merchants/me` | Yes | Current merchant profile plus FX |
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
- Privy app
- Starknet Sepolia RPC URL
- AVNU API key
- Treasury Starknet account with STRK for deployment prefunds
- Local Starkzap-rs checkout at `/Users/MAC/Rust/starkzap-rs` or update `Cargo.toml` to use a git dependency

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

## Tech Stack

| Layer | Tech |
|---|---|
| Backend | Rust, Axum, Tokio |
| Database | PostgreSQL, Diesel, diesel-async |
| Wallet SDK | Starkzap-rs |
| Starknet SDK | starknet-rs `0.17` |
| Auth & Wallets | Privy |
| Paymaster | AVNU |
| Worker | Rust binary polling Starknet events |
| Email | lettre SMTP |
| QR | `qrcode` + `image` |
| Frontend | Vite, React 19, TypeScript, Tailwind CSS v4 |