import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, vi } from 'vitest'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import AdminUsersView from '../src/views/AdminUsers/AdminUsersView'
import EditUserView from '../src/views/AdminUsers/EditUserView'

const apiMock = vi.hoisted(() => ({
  get: vi.fn(),
  patch: vi.fn(),
}))

vi.mock('../src/api/httpClient', () => ({ default: apiMock }))

const userRecord = {
  id: 'user-1',
  name: 'Usuario de prueba',
  email: 'usuario@example.com',
  role: 'Student' as const,
  status: 'active' as const,
  created_at: '2026-09-25T10:00:00Z',
  updated_at: '2026-09-25T10:00:00Z',
}

function renderAdminUsers(initialPath = '/admin/users') {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[initialPath]}>
        <Routes>
          <Route path="/admin/users" element={<AdminUsersView />} />
          <Route path="/admin/users/:userId/edit" element={<EditUserView />} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

beforeEach(() => {
  vi.clearAllMocks()
})

describe('AdminUsersView', () => {
  it('loads users with search and role filters and toggles account status', async () => {
    const user = userEvent.setup()
    apiMock.get.mockResolvedValue({ data: { items: [userRecord] } })
    apiMock.patch.mockResolvedValue({
      data: { ...userRecord, status: 'inactive' },
    })
    renderAdminUsers()

    expect(await screen.findByText('Usuario de prueba')).toBeInTheDocument()
    await user.type(screen.getByLabelText('Buscar'), 'usuario')
    await user.selectOptions(screen.getByLabelText('Rol'), 'Student')

    await waitFor(() => {
      expect(apiMock.get).toHaveBeenLastCalledWith(
        '/users?q=usuario&role=Student',
      )
    })
    await user.click(screen.getByRole('button', { name: 'Desactivar' }))

    expect(apiMock.patch).toHaveBeenCalledWith('/users/user-1/status', {
      status: 'inactive',
    })
  })
})

describe('EditUserView', () => {
  it('loads the user and patches only changed fields', async () => {
    const user = userEvent.setup()
    apiMock.get.mockImplementation((url: string) =>
      Promise.resolve({
        data: url === '/users/user-1' ? userRecord : { items: [] },
      }),
    )
    apiMock.patch.mockResolvedValue({
      data: { ...userRecord, name: 'Usuario actualizado' },
    })
    renderAdminUsers('/admin/users/user-1/edit')

    const nameField = await screen.findByLabelText('Nombre')
    await user.clear(nameField)
    await user.type(nameField, 'Usuario actualizado')
    await user.click(screen.getByRole('button', { name: 'Guardar cambios' }))

    expect(apiMock.get).toHaveBeenCalledWith('/users/user-1')
    expect(apiMock.patch).toHaveBeenCalledWith('/users/user-1', {
      name: 'Usuario actualizado',
    })
    expect(await screen.findByRole('heading', { name: 'Usuarios' }))
      .toBeInTheDocument()
  })
})
