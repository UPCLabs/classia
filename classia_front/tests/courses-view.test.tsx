import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { vi } from 'vitest'
import { AuthContext } from '../src/auth/context'
import CoursesView from '../src/views/EstudentCourses/CoursesView'

const apiMock = vi.hoisted(() => ({
  get: vi.fn(),
}))

vi.mock('../src/api/httpClient', () => ({ default: apiMock }))

describe('CoursesView', () => {
  it('loads courses and offers course creation to a teacher', async () => {
    apiMock.get.mockResolvedValue({
      data: {
        items: [
          {
            id: 'course-1',
            name: 'Programación',
            code: 'PRG101',
            teacher: {
              id: 'teacher-1',
              name: 'Docente de prueba',
              email: 'docente@example.com',
            },
            status: 'active',
            capacity: 25,
            enrolled_count: 7,
            created_at: '2026-09-25T10:00:00Z',
            updated_at: '2026-09-25T10:00:00Z',
          },
        ],
      },
    })
    const client = new QueryClient()

    render(
      <QueryClientProvider client={client}>
        <AuthContext.Provider
          value={{
            user: {
              id: 'teacher-1',
              name: 'Docente de prueba',
              email: 'docente@example.com',
              role: 'Teacher',
            },
            isLoading: false,
            error: null,
            signIn: vi.fn(),
            signOut: vi.fn(),
          }}
        >
          <MemoryRouter>
            <CoursesView />
          </MemoryRouter>
        </AuthContext.Provider>
      </QueryClientProvider>,
    )

    expect(await screen.findByText('Programación')).toBeInTheDocument()
    expect(screen.getByText('Docente: Docente de prueba')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Crear curso' })).toHaveAttribute(
      'href',
      '/courses/new',
    )
  })
})
