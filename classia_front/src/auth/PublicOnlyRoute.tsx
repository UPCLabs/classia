import type { ReactNode } from 'react'
import { Navigate } from 'react-router-dom'
import { useAuth } from './context'

type PublicOnlyRouteProps = {
  children: ReactNode
}

export default function PublicOnlyRoute({ children }: PublicOnlyRouteProps) {
  const { user, isLoading, error } = useAuth()

  if (isLoading) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-verde-oscuro text-crema">
        <p role="status">Cargando sesión...</p>
      </main>
    )
  }

  if (error) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-verde-oscuro px-4 text-crema">
        <p role="alert">{error}</p>
      </main>
    )
  }

  return user ? <Navigate replace to="/dashboard" /> : children
}
