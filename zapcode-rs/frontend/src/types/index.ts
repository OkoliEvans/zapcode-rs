export interface Merchant {
  id:            string
  email:         string
  businessName:  string
  walletAddress: string
  walletId:      string
  publicKey:     string
  currency:      string
  country:       string
  network:       'sepolia'
  logoUrl:       string | null
  isActive:      boolean
  createdAt:     string
  fx:            FxRate | null
}

export interface FxRate {
  rate:       number
  currency:   string
  fetchedAt:  number
  ageSeconds: number
}

export interface Transaction {
  id:          string
  merchantId:  string
  txHash:      string
  fromAddress: string
  toAddress:   string
  amount:      string
  currency:    string
  status:      'pending' | 'confirmed' | 'failed'
  blockNumber: string | null
  note:        string | null
  detectedAt:  string
  confirmedAt: string | null
}

export interface Stats {
  totalRevenue:  number
  orderCount:    number
  avgOrderValue: number
  todayRevenue:  number
  todayOrders:   number
  weekRevenue:   number
  weekOrders:    number
}

export interface PublicMerchant {
  id:            string
  businessName:  string
  walletAddress: string
  currency:      string
  network:       string
  logoUrl:       string | null
  fx:            FxRate | null
}

export type OnboardingData = {
  businessName: string
  currency:     string
  country:      string
  network:      'sepolia'
}
