import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { AuthContext } from '../src/auth/context'
import LoginView from '../src/views/LoginView'
import type { SignInResult } from '../src/auth/context'

function renderLogin({
  signIn = vi.fn<() => Promise<SignInResult>>().mockResolvedValue('mfa'),
  completeMfaSignIn = vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
} = {}) {
  return render(
    <AuthContext.Provider
      value={{
        user: null,
        isLoading: false,
        error: null,
        signIn,
        completeMfaSignIn,
        signOut: vi.fn(),
      }}
    >
      <MemoryRouter initialEntries={['/auth/login']}>
        <Routes>
          <Route path="/auth/login" element={<LoginView />} />
          <Route path="/dashboard" element={<p>Panel principal</p>} />
        </Routes>
      </MemoryRouter>
    </AuthContext.Provider>,
  )
}

async function submitPasswordStep(user: ReturnType<typeof userEvent.setup>) {
  await user.type(
    await screen.findByLabelText('Correo electrónico'),
    'ana@example.com',
  )
  await user.type(screen.getByLabelText('Contraseña'), 'secret')
  await user.click(screen.getByRole('button', { name: 'Iniciar sesión' }))
}

beforeEach(() => {
  vi.clearAllMocks()
})

describe('LoginView con MFA', () => {
  it('pide el código de seis dígitos después de la contraseña', async () => {
    const user = userEvent.setup()
    const completeMfaSignIn = vi.fn().mockResolvedValue(undefined)
    renderLogin({ completeMfaSignIn })

    await submitPasswordStep(user)

    expect(
      await screen.findByRole('heading', { name: 'Verificación en dos pasos' }),
    ).toBeInTheDocument()

    await user.type(screen.getByLabelText('Código de verificación'), '123456')
    await user.click(screen.getByRole('button', { name: 'Verificar código' }))

    expect(completeMfaSignIn).toHaveBeenCalledWith('123456')
    expect(await screen.findByText('Panel principal')).toBeInTheDocument()
  })

  it('rechaza un código que no tiene seis dígitos', async () => {
    const user = userEvent.setup()
    const completeMfaSignIn = vi.fn()
    renderLogin({ completeMfaSignIn })

    await submitPasswordStep(user)
    await user.type(
      await screen.findByLabelText('Código de verificación'),
      '12345',
    )
    await user.click(screen.getByRole('button', { name: 'Verificar código' }))

    expect(
      await screen.findByText('El código debe tener 6 dígitos.'),
    ).toBeInTheDocument()
    expect(completeMfaSignIn).not.toHaveBeenCalled()
  })

  it('muestra el error cuando el código es inválido o venció', async () => {
    const user = userEvent.setup()
    const completeMfaSignIn = vi
      .fn()
      .mockRejectedValue(
        new Error('Código inválido o vencido. Inténtalo de nuevo.'),
      )
    renderLogin({ completeMfaSignIn })

    await submitPasswordStep(user)
    await user.type(
      await screen.findByLabelText('Código de verificación'),
      '000000',
    )
    await user.click(screen.getByRole('button', { name: 'Verificar código' }))

    expect(
      await screen.findByText('Código inválido o vencido. Inténtalo de nuevo.'),
    ).toBeInTheDocument()
    expect(screen.queryByText('Panel principal')).not.toBeInTheDocument()
  })
})
