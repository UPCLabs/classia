import { useEffect, useState, type ReactNode } from 'react'
import httpClient from '../api/httpClient'
import { challengeMfa } from '../api/mfa'
import type { ApiError, AuthUser } from '../types/api'
import { AuthContext, type SignInResult } from './context'
import getErrorMessage from './getErrorMessage'

function isUnauthenticatedError(error: unknown): error is ApiError {
  if (typeof error !== 'object' || error === null || !('code' in error)) {
    return false
  }

  return ['401', '404', 'UNAUTHORIZED', 'INVALID_TOKEN'].includes(
    String(error.code),
  )
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

  async function signIn(email: string, password: string): Promise<SignInResult> {
    let requiresMfa: boolean

    try {
      const { data } = await httpClient.post<{ mfa_required?: boolean }>(
        '/auth/login',
        { email, password },
      )
      requiresMfa = data?.mfa_required === true
    } catch (loginError: unknown) {
      if (!isMfaRequiredError(loginError)) {
        throw loginError
      }
      requiresMfa = true
    }

    if (requiresMfa) {
      return 'mfa'
    }

    await loadCurrentUser()
    return 'authenticated'
  }

  async function completeMfaSignIn(code: string) {
    const verified = await challengeMfa(code)
    if (!verified) {
      throw new Error('Código inválido o vencido. Inténtalo de nuevo.')
    }
    await loadCurrentUser()
  }

  async function loadCurrentUser() {
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
    <AuthContext.Provider
      value={{ user, isLoading, error, signIn, completeMfaSignIn, signOut }}
    >
      {children}
    </AuthContext.Provider>
  )
}

function isMfaRequiredError(error: unknown): boolean {
  if (typeof error !== 'object' || error === null || !('code' in error)) {
    return false
  }

  return (error as ApiError).code === 'MFA_REQUIRED'
}
