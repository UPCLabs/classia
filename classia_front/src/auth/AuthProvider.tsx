import { useEffect, useState, type ReactNode } from 'react'
import httpClient from '../api/httpClient'
import type { ApiError, AuthUser } from '../types/api'
import { AuthContext } from './context'
import getErrorMessage from './getErrorMessage'

function isUnauthenticatedError(error: unknown): error is ApiError {
  if (typeof error !== 'object' || error === null || !('code' in error)) {
    return false
  }

  return ['401', 'UNAUTHORIZED', 'INVALID_TOKEN'].includes(String(error.code))
}

type AuthProviderProps = {
  children: ReactNode
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<AuthUser | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let active = true

    httpClient
      .get<AuthUser>('/auth/me')
      .then(({ data }) => {
        if (active) {
          setUser(data)
        }
      })
      .catch((requestError: unknown) => {
        if (active && !isUnauthenticatedError(requestError)) {
          setError(getErrorMessage(requestError))
        }
      })
      .finally(() => {
        if (active) {
          setIsLoading(false)
        }
      })

    return () => {
      active = false
    }
  }, [])

  async function signIn(email: string, password: string) {
    await httpClient.post<void>('/auth/login', { email, password })
    const { data } = await httpClient.get<AuthUser>('/auth/me')
    setUser(data)
    setError(null)
  }

  async function signOut() {
    await httpClient.post<void>('/auth/logout')
    setUser(null)
    setError(null)
  }

  return (
    <AuthContext.Provider value={{ user, isLoading, error, signIn, signOut }}>
      {children}
    </AuthContext.Provider>
  )
}
