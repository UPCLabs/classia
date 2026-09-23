import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import LandingPage from '../src/views/LandingPage'
import Router from '../src/router'

function renderRouter(path: string) {
  window.history.pushState({}, '', path)
  const client = new QueryClient()
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
})

describe('Router', () => {
  it.each([
    ['/auth/login', 'Iniciar sesión'],
    ['/auth/register', 'Registro de usuario'],
    ['/profile', 'Guardar cambios'],
    ['/courses', 'Mis cursos'],
    ['/', 'Aprende, enseña y crece con Classia'],
  ])('renders %s', async (path, landmark) => {
    renderRouter(path)
    expect(await screen.findByText(landmark)).toBeInTheDocument()
  })
})
