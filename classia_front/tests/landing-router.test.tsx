import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { beforeEach, vi } from 'vitest'
import LandingPage from '../src/views/LandingPage'
import Router from '../src/router'

const apiMock = vi.hoisted(() => ({
  get: vi.fn(),
  post: vi.fn(),
}))

vi.mock('../src/api/httpClient', () => ({ default: apiMock }))

const authenticatedUser = {
  id: 'user-1',
  name: 'Ana Torres',
  email: 'ana@example.com',
  role: 'Admin' as const,
}

beforeEach(() => {
  vi.clearAllMocks()
  apiMock.get.mockResolvedValue({ data: authenticatedUser })
})

function renderRouter(path: string) {
  window.history.pushState({}, '', path)
  const client = new QueryClient()
  if (path === '/courses') {
    apiMock.get
      .mockResolvedValueOnce({ data: authenticatedUser })
      .mockResolvedValueOnce({ data: { items: [] } })
  }
  return render(
    <QueryClientProvider client={client}>
      <Router />
    </QueryClientProvider>,
  )
}

describe('LandingPage', () => {
  it('renders its primary content and current links', () => {
    render(<MemoryRouter><LandingPage /></MemoryRouter>)

    expect(screen.getByRole('heading', { name: 'Aprende, enseña y crece con Classia' })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: '¿Qué es Classia?' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Comenzar' })).toHaveAttribute('href', '/auth/login')
    expect(screen.getByRole('link', { name: 'Iniciar sesión' })).toHaveAttribute('href', '/auth/login')
  })

  it('remains public when the session lookup is unauthorized', async () => {
    apiMock.get.mockRejectedValueOnce({
      code: 'UNAUTHORIZED',
      message: 'Authentication is required',
    })

    renderRouter('/')

    expect(
      await screen.findByRole('heading', {
        name: 'Aprende, enseña y crece con Classia',
      }),
    ).toBeInTheDocument()
  })
})

describe('Router', () => {
  it.each([
    ['/auth/login', 'Panel principal'],
    ['/profile', 'Guardar cambios'],
    ['/courses', 'Cursos'],
    ['/account/change-password', 'Cambiar contraseña'],
    ['/change-password', 'Cambiar contraseña'],
    ['/dashboard', 'Panel principal'],
    ['/admin/users', 'Usuarios'],
    ['/admin/users/new', 'Crear usuario'],
    ['/', 'Aprende, enseña y crece con Classia'],
  ])('renders %s', async (path, landmark) => {
    if (path.startsWith('/admin/users')) {
      apiMock.get
        .mockResolvedValueOnce({ data: authenticatedUser })
        .mockResolvedValueOnce({ data: { items: [] } })
    }
    renderRouter(path)
    expect(
      await screen.findByText(landmark, {
        selector: path.endsWith('/new') ? 'h1' : undefined,
      }),
    ).toBeInTheDocument()
  })

  it('restricts user administration to administrator roles', async () => {
    apiMock.get.mockResolvedValueOnce({
      data: { ...authenticatedUser, role: 'Student' },
    })

    renderRouter('/admin/users')

    expect(await screen.findByRole('heading', { name: 'Panel principal' }))
      .toBeInTheDocument()
  })

  it('shows direct navigation to home, courses and administration for admins', async () => {
    renderRouter('/dashboard')

    const navigation = await screen.findByRole('navigation', {
      name: 'Navegación principal',
    })
    const homeLink = screen.getByRole('link', { name: 'Inicio' })
    const coursesLink = screen.getByRole('link', { name: 'Cursos' })
    const adminLink = screen.getByRole('link', { name: 'Administración' })
    expect(homeLink).toHaveAttribute('href', '/dashboard')
    expect(coursesLink).toHaveAttribute('href', '/courses')
    expect(adminLink).toHaveAttribute('href', '/admin/users')
    expect(navigation).toContainElement(
      adminLink,
    )
  })

  it('shows direct navigation to home and courses for teachers without user admin', async () => {
    apiMock.get.mockResolvedValueOnce({
      data: { ...authenticatedUser, role: 'Teacher' },
    })

    renderRouter('/dashboard')

    await screen.findByRole('navigation', { name: 'Navegación principal' })
    expect(screen.getByRole('link', { name: 'Inicio' })).toHaveAttribute(
      'href',
      '/dashboard',
    )
    expect(screen.getByRole('link', { name: 'Cursos' })).toHaveAttribute(
      'href',
      '/courses',
    )
    expect(
      screen.queryByRole('link', { name: 'Administración' }),
    ).not.toBeInTheDocument()
  })

  it('redirects protected routes to login without a session', async () => {
    apiMock.get.mockRejectedValueOnce({
      code: 'UNAUTHORIZED',
      message: 'Authentication is required',
    })

    renderRouter('/courses')

    expect(await screen.findByLabelText('Correo electrónico')).toBeInTheDocument()
  })
})
