import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { vi } from 'vitest'
import ProfileView from '../src/views/ProfileView'

function renderProfile() {
  const client = new QueryClient({ defaultOptions: { mutations: { retry: false } } })
  return render(
    <QueryClientProvider client={client}>
      <ProfileView />
    </QueryClientProvider>,
  )
}

describe('ProfileView', () => {
  it('validates required fields and invalid email', async () => {
    const user = userEvent.setup()
    renderProfile()

    await user.click(screen.getByRole('button', { name: 'Guardar cambios' }))
    expect(await screen.findByText('El nombre es obligatorio')).toBeInTheDocument()
    expect(screen.getByText('El correo es obligatorio')).toBeInTheDocument()
    expect(screen.getByText('La carrera es obligatoria')).toBeInTheDocument()
    expect(screen.getByText('La descripcion es obligatoria')).toBeInTheDocument()

    await user.type(screen.getByLabelText('Correo'), 'correo-invalido')
    await user.click(screen.getByRole('button', { name: 'Guardar cambios' }))
    expect(await screen.findByText('Ingresa un correo valido')).toBeInTheDocument()
  })

  it('cancels local edits and submits the local mutation', async () => {
    const user = userEvent.setup()
    const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => undefined)
    renderProfile()

    const name = screen.getByLabelText('Nombre')
    await user.type(name, 'Temporal')
    await user.click(screen.getByRole('button', { name: 'Cancelar' }))
    expect(name).toHaveValue('')
    await user.type(name, 'Ana')
    await user.type(screen.getByLabelText('Correo'), 'ana@example.com')
    await user.type(screen.getByLabelText('Carrera'), 'Sistemas')
    await user.type(screen.getByLabelText('Descripcion'), 'Estudiante')
    await user.click(screen.getByRole('button', { name: 'Guardar cambios' }))

    await waitFor(() => expect(consoleSpy).toHaveBeenCalled())
    expect(consoleSpy.mock.calls.at(-1)?.[0]).toEqual(expect.objectContaining({
      name: 'Ana', email: 'ana@example.com', career: 'Sistemas', description: 'Estudiante',
    }))
    consoleSpy.mockRestore()
  })
})
