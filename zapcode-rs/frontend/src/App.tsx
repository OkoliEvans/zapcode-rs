import { BrowserRouter, Routes, Route, Navigate, Outlet } from 'react-router-dom'
import { usePrivy } from '@privy-io/react-auth'
import { useMerchant } from './context/MerchantContext'
import { DashboardLayout } from './components/layout/DashboardLayout'
import { LandingPage }    from './pages/LandingPage'
import { OnboardingPage } from './pages/OnboardingPage'
import { OverviewPage }   from './pages/OverviewPage'
import { PaymentsPage }   from './pages/PaymentsPage'
import { QRPage }         from './pages/QRPage'
import { SettingsPage }   from './pages/SettingsPage'
import { PayPage }        from './pages/PayPage'
import { Spinner }        from './components/ui'

// Requires Privy auth + completed onboarding
function ProtectedRoute() {
  const { ready, authenticated } = usePrivy()
  const { merchant, loading }    = useMerchant()

  if (!ready || loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <Spinner size={28} />
      </div>
    )
  }
  if (!authenticated)  return <Navigate to="/" replace />
  if (!merchant)       return <Navigate to="/onboard" replace />
  return <Outlet />
}

// Requires auth but no onboarding yet
function OnboardingRoute() {
  const { ready, authenticated } = usePrivy()
  const { merchant, loading }    = useMerchant()

  if (!ready || loading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <Spinner size={28} />
      </div>
    )
  }
  if (!authenticated)  return <Navigate to="/" replace />
  if (merchant)        return <Navigate to="/dashboard" replace />
  return <Outlet />
}

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        {/* Public */}
        <Route path="/"                  element={<LandingPage />} />
        <Route path="/pay/:merchantId"   element={<PayPage />} />

        {/* Onboarding (auth required, not yet onboarded) */}
        <Route element={<OnboardingRoute />}>
          <Route path="/onboard"         element={<OnboardingPage />} />
        </Route>

        {/* Dashboard (auth + onboarded required) */}
        <Route element={<ProtectedRoute />}>
          <Route element={<DashboardLayout />}>
            <Route path="/dashboard"           element={<OverviewPage />} />
            <Route path="/dashboard/payments"  element={<PaymentsPage />} />
            <Route path="/dashboard/qr"        element={<QRPage />} />
            <Route path="/dashboard/settings"  element={<SettingsPage />} />
          </Route>
        </Route>

        <Route path="*" element={<Navigate to="/" replace />} />
      </Routes>
    </BrowserRouter>
  )
}