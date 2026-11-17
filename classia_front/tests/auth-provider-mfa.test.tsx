import { useState } from 'react'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { AuthProvider } from '../src/auth/AuthProvider'
import { useAuth } from '../src/auth/context'

const apiMock = vi.hoisted(() => ({
  get: vi.fn(),
  post: vi.fn(),
}))

vi.mock('../src/api/httpClient', () => ({ default: apiMock }))

function Consumer() {
  const { user, signIn, completeMfaSignIn } = useAuth()
  const [result, setResult] = useState('')

  return (
    <div>
      <button
        onClick={() => {
          void signIn('ana@example.com', 'secret').then(setResult)
        }}
        type="button"
      >
        Entrar
      </button>
      <button
        onClick={() => {
          void completeMfaSignIn('123456')
            .then(() => setResult('mfa-ok'))
            .catch((error: Error) => setResult(error.message))
        }}
        type="button"
      >
        Verificar
      </button>
      <p>Resultado: {result}</p>
      <p>Usuario: {user?.email ?? 'sin sesión'}</p>
    </div>
  )
}

function renderProvider() {
  return render(
    <AuthProvider>
      <Consumer />
    </AuthProvider>,
  )
}

beforeEach(() => {
  vi.clearAllMocks()
})

describe('AuthProvider con MFA', () => {
  it('devuelve "mfa" cuando el login pide el segundo paso', async () => {
    const user = userEvent.setup()
    apiMock.get.mockRejectedValue({
      code: 'UNAUTHORIZED',
      message: 'Authentication is required',
    })
    apiMock.post.mockResolvedValue({ data: { mfa_required: true } })
    renderProvider()

    await user.click(await screen.findByRole('button', { name: 'Entrar' }))

    await waitFor(() =>
      expect(screen.getByText('Resultado: mfa')).toBeInTheDocument(),
    )
    expect(apiMock.get).toHaveBeenCalledTimes(1)
  })

  it('devuelve "mfa" cuando el login responde MFA_REQUIRED', async () => {
    const user = userEvent.setup()
    apiMock.get.mockRejectedValue({
      code: 'UNAUTHORIZED',
      message: 'Authentication is required',
    })
    apiMock.post.mockRejectedValue({
      code: 'MFA_REQUIRED',
      message: 'Se requiere el código de verificación',
    })
    renderProvider()

    await user.click(await screen.findByRole('button', { name: 'Entrar' }))

    await waitFor(() =>
      expect(screen.getByText('Resultado: mfa')).toBeInTheDocument(),
    )
  })

  it('completa la sesión tras verificar el código', async () => {
    const user = userEvent.setup()
    apiMock.get
      .mockRejectedValueOnce({
        code: 'UNAUTHORIZED',
        message: 'Authentication is required',
      })
      .mockResolvedValueOnce({
        data: {
          id: 'user-1',
          name: 'Ana Torres',
          email: 'ana@example.com',
          role: 'Student',
        },
      })
    apiMock.post.mockResolvedValue({ data: { verified: true } })
    renderProvider()

    await user.click(
      await screen.findByRole('button', { name: 'Verificar' }),
    )

    await waitFor(() =>
      expect(screen.getByText('Resultado: mfa-ok')).toBeInTheDocument(),
    )
    expect(screen.getByText('Usuario: ana@example.com')).toBeInTheDocument()
  })

  it('rechaza el código inválido sin abrir sesión', async () => {
    const user = userEvent.setup()
    apiMock.get.mockRejectedValue({
      code: 'UNAUTHORIZED',
      message: 'Authentication is required',
    })
    apiMock.post.mockResolvedValue({ data: { verified: false } })
    renderProvider()

    await user.click(
      await screen.findByRole('button', { name: 'Verificar' }),
    )

    await waitFor(() =>
      expect(
        screen.getByText(
          'Resultado: Código inválido o vencido. Inténtalo de nuevo.',
        ),
      ).toBeInTheDocument(),
    )
    expect(screen.getByText('Usuario: sin sesión')).toBeInTheDocument()
  })
})
