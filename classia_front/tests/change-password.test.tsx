import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, vi } from 'vitest'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { AuthContext } from '../src/auth/context'
import ChangePasswordView from '../src/views/Account/ChangePasswordView'

const apiMock = vi.hoisted(() => ({
  patch: vi.fn(),
}))

vi.mock('../src/api/httpClient', () => ({ default: apiMock }))

const signOut = vi.fn()

function renderChangePassword() {
  return render(
    <AuthContext.Provider
      value={{
        user: {
          id: 'user-1',
          name: 'Ana Torres',
          email: 'ana@example.com',
          role: 'Student',
        },
        isLoading: false,
        error: null,
        signIn: vi.fn(),
        signOut,
      }}
    >
      <MemoryRouter initialEntries={['/account/change-password']}>
        <Routes>
          <Route
            path="/account/change-password"
            element={<ChangePasswordView />}
          />
          <Route path="/auth/login" element={<h1>Iniciar sesión</h1>} />
        </Routes>
      </MemoryRouter>
    </AuthContext.Provider>,
  )
}

beforeEach(() => {
  vi.clearAllMocks()
})

describe('ChangePasswordView', () => {
  it('changes the password, signs out, and redirects to login', async () => {
    const user = userEvent.setup()
    apiMock.patch.mockResolvedValue({ status: 204 })
    signOut.mockResolvedValue(undefined)
    renderChangePassword()

    await user.type(screen.getByLabelText('Nueva contraseña'), 'nueva-clave')
    await user.type(
      screen.getByLabelText('Confirmar contraseña'),
      'nueva-clave',
    )
    await user.click(screen.getByRole('button', { name: 'Guardar cambios' }))

    expect(apiMock.patch).toHaveBeenCalledWith('/users/change-password', {
      new_password: 'nueva-clave',
    })
    expect(signOut).toHaveBeenCalledOnce()
    expect(
      await screen.findByRole('heading', { name: 'Iniciar sesión' }),
    ).toBeInTheDocument()
  })

  it('validates matching passwords before calling the API', async () => {
    const user = userEvent.setup()
    renderChangePassword()

    await user.type(screen.getByLabelText('Nueva contraseña'), 'nueva-clave')
    await user.type(
      screen.getByLabelText('Confirmar contraseña'),
      'otra-clave',
    )
    await user.click(screen.getByRole('button', { name: 'Guardar cambios' }))

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Las contraseñas no coinciden',
    )
    expect(apiMock.patch).not.toHaveBeenCalled()
  })

  it('shows errors returned by the API', async () => {
    const user = userEvent.setup()
    apiMock.patch.mockRejectedValue({
      code: 'VALIDATION_ERROR',
      message: 'La contraseña no cumple los requisitos',
    })
    renderChangePassword()

    await user.type(screen.getByLabelText('Nueva contraseña'), 'nueva-clave')
    await user.type(
      screen.getByLabelText('Confirmar contraseña'),
      'nueva-clave',
    )
    await user.click(screen.getByRole('button', { name: 'Guardar cambios' }))

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'La contraseña no cumple los requisitos',
    )
    expect(signOut).not.toHaveBeenCalled()
  })
})
