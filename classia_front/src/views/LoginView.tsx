import { useState, type FormEvent } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import { useAuth } from '../auth/context'
import getErrorMessage from '../auth/getErrorMessage'

type LoginLocationState = {
  from?: {
    pathname?: string
  }
}

export default function LoginView() {
  const { signIn } = useAuth()
  const navigate = useNavigate()
  const location = useLocation()
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function handleSubmit(e: FormEvent<HTMLFormElement>) {
    e.preventDefault()
    setIsSubmitting(true)
    setError(null)

    try {
      await signIn(email, password)
      const state = location.state as LoginLocationState | null
      const returnPath = state?.from?.pathname
      const destination =
        returnPath?.startsWith('/') && !returnPath.startsWith('//')
          ? returnPath
          : '/dashboard'
      navigate(destination, { replace: true })
    } catch (loginError: unknown) {
      setError(getErrorMessage(loginError))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <main className="flex min-h-screen items-center justify-center bg-verde-oscuro px-4">
      <div className="w-full max-w-sm rounded-2xl bg-verde-medio p-8 shadow-xl">
        <h1 className="mb-6 text-center text-2xl font-bold text-crema">
          Iniciar sesión
        </h1>

        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div className="flex flex-col gap-1">
            <label className="text-sm font-medium text-crema" htmlFor="login-email">
              Correo electrónico
            </label>
            <input
              autoComplete="email"
              className="rounded-lg bg-crema px-3 py-2 text-verde-oscuro outline-none focus:ring-2 focus:ring-dorado"
              id="login-email"
              onChange={(e) => setEmail(e.target.value)}
              required
              type="email"
              value={email}
            />
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-sm font-medium text-crema" htmlFor="login-password">
              Contraseña
            </label>
            <input
              autoComplete="current-password"
              className="rounded-lg bg-crema px-3 py-2 text-verde-oscuro outline-none focus:ring-2 focus:ring-dorado"
              id="login-password"
              onChange={(e) => setPassword(e.target.value)}
              required
              type="password"
              value={password}
            />
          </div>

          {error && (
            <p className="rounded-lg bg-crema px-3 py-2 text-sm text-red-800" role="alert">
              {error}
            </p>
          )}

          <button
            className="mt-2 rounded-lg bg-dorado py-2 font-semibold text-verde-oscuro transition hover:brightness-110 disabled:cursor-wait disabled:opacity-70"
            disabled={isSubmitting}
            type="submit"
          >
            {isSubmitting ? 'Ingresando...' : 'Iniciar sesión'}
          </button>
        </form>
      </div>
    </main>
  )
}
