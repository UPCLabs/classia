import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, vi } from 'vitest'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { AuthProvider } from '../src/auth/AuthProvider'
import AuthenticatedLayout from '../src/layouts/AuthenticatedLayout'
import DashboardView from '../src/views/DashboardView'
import LoginView from '../src/views/LoginView'
import RegisterView from '../src/views/RegisterView'

const apiMock = vi.hoisted(() => ({
  get: vi.fn(),
  post: vi.fn(),
}))

vi.mock('../src/api/httpClient', () => ({ default: apiMock }))

const authenticatedUser = {
  id: 'user-1',
  name: 'Ana Torres',
  email: 'ana@example.com',
  role: 'Student' as const,
}

function renderLogin() {
  return render(
    <MemoryRouter initialEntries={['/auth/login']}>
      <AuthProvider>
        <Routes>
          <Route path="/auth/login" element={<LoginView />} />
          <Route element={<AuthenticatedLayout />}>
            <Route path="/dashboard" element={<DashboardView />} />
          </Route>
        </Routes>
      </AuthProvider>
    </MemoryRouter>,
  )
}

beforeEach(() => {
  vi.clearAllMocks()
})

describe('LoginView', () => {
  it('signs in through the API and opens the dashboard', async () => {
    const user = userEvent.setup()
    apiMock.get
      .mockRejectedValueOnce({
        code: 'UNAUTHORIZED',
        message: 'Authentication is required',
      })
      .mockResolvedValueOnce({ data: authenticatedUser })
    apiMock.post.mockResolvedValue({ status: 204 })
    renderLogin()

    await user.type(screen.getByLabelText('Correo electrónico'), 'ana@example.com')
    await user.type(screen.getByLabelText('Contraseña'), 'secret')
    await user.click(screen.getByRole('button', { name: 'Iniciar sesión' }))

    expect(apiMock.post).toHaveBeenCalledWith('/auth/login', {
      email: 'ana@example.com',
      password: 'secret',
    })
    expect(await screen.findByRole('heading', { name: 'Panel principal' }))
      .toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Abrir menú de ajustes' }))
    expect(screen.getByRole('link', { name: 'Cambiar contraseña' })).toHaveAttribute(
      'href',
      '/account/change-password',
    )
  })

  it('shows an API error when the credentials are rejected', async () => {
    const user = userEvent.setup()
    apiMock.get.mockRejectedValueOnce({
      code: 'UNAUTHORIZED',
      message: 'Authentication is required',
    })
    apiMock.post.mockRejectedValue({
      code: '401',
      message: 'Credenciales inválidas',
    })
    renderLogin()

    await user.type(screen.getByLabelText('Correo electrónico'), 'ana@example.com')
    await user.type(screen.getByLabelText('Contraseña'), 'incorrecta')
    await user.click(screen.getByRole('button', { name: 'Iniciar sesión' }))

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Credenciales inválidas',
    )
  })

  it('clears the session after a successful logout', async () => {
    const user = userEvent.setup()
    apiMock.get
      .mockRejectedValueOnce({
        code: 'UNAUTHORIZED',
        message: 'Authentication is required',
      })
      .mockResolvedValueOnce({ data: authenticatedUser })
    apiMock.post.mockResolvedValue({ status: 204 })
    renderLogin()

    await user.type(screen.getByLabelText('Correo electrónico'), 'ana@example.com')
    await user.type(screen.getByLabelText('Contraseña'), 'secret')
    await user.click(screen.getByRole('button', { name: 'Iniciar sesión' }))
    await user.click(await screen.findByRole('button', { name: 'Abrir menú de ajustes' }))
    await user.click(screen.getByRole('button', { name: 'Cerrar sesión' }))

    expect(apiMock.post).toHaveBeenNthCalledWith(2, '/auth/logout')
    expect(await screen.findByLabelText('Correo electrónico')).toBeInTheDocument()
  })
})

describe('RegisterView', () => {
  it('edits and submits the registration fields', async () => {
    const user = userEvent.setup()
    const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => undefined)
    render(<RegisterView />)

    await user.type(screen.getByLabelText('Nombre completo'), 'Ana Torres')
    await user.type(screen.getByLabelText('Correo'), 'ana@example.com')
    await user.type(screen.getByLabelText('Código'), 'A001')
    await user.type(screen.getByLabelText('Carrera'), 'Sistemas')
    await user.click(screen.getByRole('button', { name: 'Registrarse' }))

    expect(consoleSpy).toHaveBeenCalledWith('Register with:', {
      code: 'A001',
      name: 'Ana Torres',
      email: 'ana@example.com',
      career: 'Sistemas',
    })
    consoleSpy.mockRestore()
  })
})
