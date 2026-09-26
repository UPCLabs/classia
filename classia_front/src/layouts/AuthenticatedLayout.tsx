import { useState } from 'react'
import { Link, Outlet, useNavigate } from 'react-router-dom'
import { useAuth } from '../auth/context'
import getErrorMessage from '../auth/getErrorMessage'

export default function AuthenticatedLayout() {
  const { user, signOut } = useAuth()
  const navigate = useNavigate()
  const [isMenuOpen, setIsMenuOpen] = useState(false)
  const [isSigningOut, setIsSigningOut] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function handleSignOut() {
    setIsSigningOut(true)
    setError(null)

    try {
      await signOut()
      navigate('/auth/login', { replace: true })
    } catch (signOutError: unknown) {
      setError(getErrorMessage(signOutError))
    } finally {
      setIsSigningOut(false)
    }
  }

  return (
    <div className="min-h-screen bg-crema text-verde-oscuro">
      <header className="flex items-center justify-between border-b-2 border-dorado bg-verde-oscuro px-6 py-5 text-crema">
        <Link className="text-xl font-bold" to="/dashboard">
          Classia
        </Link>

        <div className="relative">
          <button
            aria-controls="account-settings-menu"
            aria-expanded={isMenuOpen}
            aria-label={isMenuOpen ? 'Cerrar menú de ajustes' : 'Abrir menú de ajustes'}
            className="flex items-center gap-2 rounded-lg border border-dorado px-4 py-2 font-semibold transition hover:bg-dorado hover:text-verde-oscuro focus:outline-none focus:ring-2 focus:ring-dorado"
            onClick={() => setIsMenuOpen((open) => !open)}
            type="button"
          >
            <svg
              aria-hidden="true"
              className="h-5 w-5"
              fill="none"
              stroke="currentColor"
              strokeLinecap="round"
              strokeWidth="2"
              viewBox="0 0 24 24"
            >
              {isMenuOpen ? (
                <path d="M6 6l12 12M18 6L6 18" />
              ) : (
                <path d="M4 6h16M4 12h16M4 18h16" />
              )}
            </svg>
            Ajustes
          </button>

          {isMenuOpen && (
            <nav
              aria-label="Ajustes de cuenta"
              className="absolute right-0 z-10 mt-2 w-64 rounded-xl border border-dorado bg-white p-2 text-verde-oscuro shadow-xl"
              id="account-settings-menu"
            >
              {user && (
                <p className="border-b border-verde-oscuro/10 px-3 py-2 text-sm text-verde-oscuro/70">
                  {user.name}
                </p>
              )}
              <Link
                className="block rounded-lg px-3 py-2 font-medium hover:bg-crema"
                onClick={() => setIsMenuOpen(false)}
                to="/account/change-password"
              >
                Cambiar contraseña
              </Link>
              <button
                className="w-full rounded-lg px-3 py-2 text-left font-medium text-red-800 hover:bg-red-50 disabled:opacity-60"
                disabled={isSigningOut}
                onClick={handleSignOut}
                type="button"
              >
                {isSigningOut ? 'Cerrando sesión...' : 'Cerrar sesión'}
              </button>
            </nav>
          )}
        </div>
      </header>

      {error && (
        <p
          className="mx-auto mt-4 max-w-5xl rounded-lg border border-red-700 bg-white p-4 text-red-800"
          role="alert"
        >
          {error}
        </p>
      )}

      <Outlet />
    </div>
  )
}
