-- Create custom types
CREATE TYPE tx_status AS ENUM ('pending', 'confirmed', 'failed');

-- Create merchants table
CREATE TABLE IF NOT EXISTS merchants (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    business_name TEXT NOT NULL,
    wallet_id TEXT NOT NULL,
    wallet_address TEXT NOT NULL UNIQUE,
    public_key TEXT NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    country VARCHAR(2) NOT NULL DEFAULT 'KE',
    network TEXT NOT NULL DEFAULT 'sepolia',
    logo_url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create transactions table
CREATE TABLE IF NOT EXISTS transactions (
    id TEXT PRIMARY KEY,
    merchant_id TEXT NOT NULL REFERENCES merchants(id),
    tx_hash TEXT NOT NULL UNIQUE,
    from_address TEXT NOT NULL,
    to_address TEXT NOT NULL,
    amount DECIMAL(28, 6) NOT NULL,
    currency VARCHAR(10) NOT NULL DEFAULT 'USDC',
    status tx_status NOT NULL DEFAULT 'pending',
    block_number TEXT,
    note TEXT,
    email_sent BOOLEAN NOT NULL DEFAULT false,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    confirmed_at TIMESTAMPTZ
);

-- Create buyers table
CREATE TABLE IF NOT EXISTS buyers (
    id TEXT PRIMARY KEY,
    wallet_id TEXT NOT NULL,
    wallet_address TEXT NOT NULL UNIQUE,
    public_key TEXT NOT NULL,
    network TEXT NOT NULL DEFAULT 'mainnet',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX IF NOT EXISTS merchant_wallet_address_idx ON merchants(wallet_address);
CREATE INDEX IF NOT EXISTS tx_merchant_id_idx ON transactions(merchant_id);
CREATE INDEX IF NOT EXISTS tx_hash_idx ON transactions(tx_hash);
CREATE INDEX IF NOT EXISTS tx_detected_at_idx ON transactions(detected_at);
CREATE INDEX IF NOT EXISTS buyer_wallet_address_idx ON buyers(wallet_address);