import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { PrivyProvider } from '@privy-io/react-auth'
import { MerchantProvider } from './context/MerchantContext'
import { ToastProvider } from './context/ToastContext'
import './index.css'
import App from './App'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <PrivyProvider
      appId={import.meta.env.VITE_PRIVY_APP_ID as string}
      config={{
        // Both merchants and buyers login with email or Google
        loginMethods: ['email', 'google'],
        appearance: {
          theme:       'dark',
          accentColor: '#c8922a',
        },
        // Privy manages Starknet wallets server-side — suppress client-side wallet UIs
        embeddedWallets: { showWalletUIs: false },
      }}
    >
      <MerchantProvider>
        <ToastProvider>
          <App />
        </ToastProvider>
      </MerchantProvider>
    </PrivyProvider>
  </StrictMode>,
)