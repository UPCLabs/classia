import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { AuthContext } from '../src/auth/context'
import AdminUsersView from '../src/views/AdminUsers/AdminUsersView'
import type { UserRole, Usuario } from '../src/types/api'

const apiMock = vi.hoisted(() => ({
  get: vi.fn(),
  patch: vi.fn(),
  post: vi.fn(),
}))

vi.mock('../src/api/httpClient', () => ({ default: apiMock }))

function userRecord(
  id: string,
  name: string,
  role: UserRole,
): Usuario {
  return {
    id,
    name,
    email: `${id}@example.com`,
    role,
    status: 'active',
    created_at: '2026-09-25T10:00:00Z',
    updated_at: '2026-09-25T10:00:00Z',
  }
}

const teacher = userRecord('teacher-1', 'Docente Prueba', 'Teacher')
const student = userRecord('student-1', 'Estudiante Prueba', 'Student')
const otherAdmin = userRecord('admin-2', 'Otro Admin', 'Admin')

function renderAdmin(currentRole: UserRole, currentId = 'admin-1') {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <AuthContext.Provider
        value={{
          user: {
            id: currentId,
            name: 'Cuenta actual',
            email: 'actual@example.com',
            role: currentRole,
          },
          isLoading: false,
          error: null,
          signIn: vi.fn(),
          completeMfaSignIn: vi.fn(),
          signOut: vi.fn(),
        }}
      >
        <MemoryRouter initialEntries={['/admin/users']}>
          <Routes>
            <Route path="/admin/users" element={<AdminUsersView />} />
          </Routes>
        </MemoryRouter>
      </AuthContext.Provider>
    </QueryClientProvider>,
  )
}

beforeEach(() => {
  vi.clearAllMocks()
})

describe('AdminUsersView — restablecer MFA', () => {
  it('el administrador solo puede restablecer docentes y estudiantes', async () => {
    apiMock.get.mockResolvedValue({
      data: {
        items: [
          userRecord('admin-1', 'Cuenta actual', 'Admin'),
          otherAdmin,
          teacher,
          student,
        ],
      },
    })
    renderAdmin('Admin')

    await screen.findByText('Docente Prueba')
    expect(
      screen.getAllByRole('button', { name: 'Restablecer MFA' }),
    ).toHaveLength(2)
  })

  it('el superadministrador solo restablece administradores', async () => {
    apiMock.get.mockResolvedValue({
      data: {
        items: [
          userRecord('super-1', 'Cuenta actual', 'SuperAdmin'),
          otherAdmin,
          teacher,
        ],
      },
    })
    renderAdmin('SuperAdmin', 'super-1')

    await screen.findByText('Otro Admin')
    expect(
      screen.getAllByRole('button', { name: 'Restablecer MFA' }),
    ).toHaveLength(1)
  })

  it('exige motivo y envía el restablecimiento', async () => {
    const user = userEvent.setup()
    apiMock.get.mockResolvedValue({ data: { items: [teacher, student] } })
    apiMock.post.mockResolvedValue({ data: undefined })
    renderAdmin('Admin')

    await user.click(
      (await screen.findAllByRole('button', { name: 'Restablecer MFA' }))[0],
    )

    expect(
      screen.getByRole('heading', { name: 'Restablecer MFA' }),
    ).toBeInTheDocument()
    const confirm = screen.getByRole('button', {
      name: 'Confirmar restablecimiento',
    })
    expect(confirm).toBeDisabled()

    await user.type(
      screen.getByLabelText('Motivo (obligatorio)'),
      'Perdió el teléfono',
    )
    expect(confirm).toBeEnabled()
    await user.click(confirm)

    await waitFor(() =>
      expect(apiMock.post).toHaveBeenCalledWith(
        '/users/teacher-1/reset-mfa',
        { reason: 'Perdió el teléfono' },
      ),
    )
    expect(
      await screen.findByText('Se restableció el MFA de Docente Prueba.'),
    ).toBeInTheDocument()
    expect(
      screen.queryByRole('heading', { name: 'Restablecer MFA' }),
    ).not.toBeInTheDocument()
  })

  it('nadie puede restablecerse el MFA a sí mismo', async () => {
    apiMock.get.mockResolvedValue({
      data: {
        items: [userRecord('admin-1', 'Cuenta actual', 'Admin')],
      },
    })
    renderAdmin('Admin')

    await screen.findByText('Cuenta actual')
    expect(
      screen.queryByRole('button', { name: 'Restablecer MFA' }),
    ).not.toBeInTheDocument()
  })
})
