import type { ReactNode } from 'react'
import { Navigate, useLocation } from 'react-router-dom'
import { useAuth } from './context'
import type { UserRole } from '../types/api'

type ProtectedRouteProps = {
  children: ReactNode
  allowedRoles?: UserRole[]
}

export default function ProtectedRoute({
  children,
  allowedRoles,
}: ProtectedRouteProps) {
  const { user, isLoading, error } = useAuth()
  const location = useLocation()

  if (isLoading) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-verde-oscuro text-crema">
        <p role="status">Cargando sesión...</p>
      </main>
    )
  }

  if (error) {
    return (
      <main className="flex min-h-screen flex-col items-center justify-center gap-4 bg-verde-oscuro px-4 text-center text-crema">
        <p role="alert">{error}</p>
        <button
          className="rounded-lg bg-dorado px-5 py-2 font-semibold text-verde-oscuro"
          onClick={() => window.location.reload()}
          type="button"
        >
          Reintentar
        </button>
      </main>
    )
  }

  if (!user) {
    return <Navigate replace state={{ from: location }} to="/auth/login" />
  }

  if (allowedRoles && !allowedRoles.includes(user.role)) {
    return <Navigate replace to="/dashboard" />
  }

  return children
}
