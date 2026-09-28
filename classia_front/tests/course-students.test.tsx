import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, vi } from 'vitest'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { AuthContext } from '../src/auth/context'
import CourseStudentsView from '../src/views/StudentCourses/CourseStudentsView'

const apiMock = vi.hoisted(() => ({
  get: vi.fn(),
  post: vi.fn(),
}))

vi.mock('../src/api/httpClient', () => ({ default: apiMock }))

const course = {
  id: 'course-1',
  name: 'Programación',
  code: 'PRG101',
  teacher: {
    id: 'teacher-1',
    name: 'Docente de prueba',
    email: 'docente@example.com',
  },
  status: 'active' as const,
  capacity: 25,
  enrolled_count: 1,
  created_at: '2026-09-25T10:00:00Z',
  updated_at: '2026-09-25T10:00:00Z',
}

const enrolledStudent = {
  id: 'student-1',
  name: 'Estudiante inscrito',
  email: 'inscrito@example.com',
  role: 'Student' as const,
  status: 'active' as const,
  created_at: '2026-09-25T10:00:00Z',
  updated_at: '2026-09-25T10:00:00Z',
}

const availableStudent = {
  id: 'student-2',
  name: 'Ana Torres',
  email: 'ana@example.com',
  role: 'Student' as const,
  status: 'active' as const,
  created_at: '2026-09-25T10:00:00Z',
  updated_at: '2026-09-25T10:00:00Z',
}

function renderParticipants(role: 'Teacher' | 'Admin' | 'SuperAdmin' = 'Teacher') {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <AuthContext.Provider
        value={{
          user: {
            id: role === 'Teacher' ? 'teacher-1' : 'admin-1',
            name: 'Cuenta de prueba',
            email: 'cuenta@example.com',
            role,
          },
          isLoading: false,
          error: null,
          signIn: vi.fn(),
          signOut: vi.fn(),
        }}
      >
        <MemoryRouter initialEntries={['/courses/course-1/students']}>
          <Routes>
            <Route
              path="/courses/:courseId/students"
              element={<CourseStudentsView />}
            />
          </Routes>
        </MemoryRouter>
      </AuthContext.Provider>
    </QueryClientProvider>,
  )
}

function mockParticipantQueries() {
  apiMock.get.mockImplementation((url: string) => {
    if (url === '/courses/course-1') {
      return Promise.resolve({ data: course })
    }
    if (url === '/courses/course-1/students') {
      return Promise.resolve({ data: { items: [enrolledStudent] } })
    }
    if (url.startsWith('/courses/course-1/available-students')) {
      return Promise.resolve({ data: { items: [availableStudent] } })
    }
    throw new Error(`Unexpected GET ${url}`)
  })
}

beforeEach(() => {
  vi.clearAllMocks()
})

describe('CourseStudentsView', () => {
  it('shows enrolled and available students to the responsible teacher', async () => {
    const user = userEvent.setup()
    mockParticipantQueries()
    renderParticipants()

    expect(await screen.findByText('Estudiante inscrito')).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: 'Agregar estudiante' }))

    expect(
      await screen.findByText('Ana Torres'),
    ).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Inscribir' })).toBeEnabled()
  })

  it('searches available students with the entered query', async () => {
    const user = userEvent.setup()
    mockParticipantQueries()
    renderParticipants()

    await user.click(
      await screen.findByRole('button', { name: 'Agregar estudiante' }),
    )
    const search = screen.getByRole('searchbox', {
      name: 'Buscar estudiantes disponibles',
    })
    await user.type(search, 'Ana')

    await waitFor(() => {
      expect(apiMock.get).toHaveBeenCalledWith(
        '/courses/course-1/available-students?q=Ana',
      )
    })
    expect(await screen.findByText('Ana Torres')).toBeInTheDocument()
  })

  it('enrolls a selected student and refreshes participants', async () => {
    const user = userEvent.setup()
    let studentListRequests = 0
    let enrollmentCompleted = false
    mockParticipantQueries()
    apiMock.get.mockImplementation((url: string) => {
      if (url === '/courses/course-1') {
        return Promise.resolve({ data: course })
      }
      if (url === '/courses/course-1/students') {
        studentListRequests += 1
        return Promise.resolve({
          data: { items: studentListRequests === 1 ? [] : [availableStudent] },
        })
      }
      if (url.startsWith('/courses/course-1/available-students')) {
        return Promise.resolve({
          data: { items: enrollmentCompleted ? [] : [availableStudent] },
        })
      }
      throw new Error(`Unexpected GET ${url}`)
    })
    apiMock.post.mockImplementation(() => {
      enrollmentCompleted = true
      return Promise.resolve({
        data: {
        course_id: 'course-1',
        student: availableStudent,
        enrolled_at: '2026-09-25T10:00:00Z',
        },
      })
    })
    renderParticipants()

    await user.click(
      await screen.findByRole('button', { name: 'Agregar estudiante' }),
    )
    await user.click(await screen.findByRole('button', { name: 'Inscribir' }))

    expect(apiMock.post).toHaveBeenCalledWith('/courses/course-1/students', {
      student_id: 'student-2',
    })
    expect(
      await screen.findByText('Estudiante inscrito correctamente.'),
    ).toHaveAttribute('role', 'status')
    expect(await screen.findByText('Ana Torres')).toBeInTheDocument()
    expect(studentListRequests).toBeGreaterThan(1)
    expect(await screen.findByText('No hay estudiantes disponibles.')).toBeInTheDocument()
  })

  it('shows a capacity error and leaves enrollment available when full', async () => {
    const user = userEvent.setup()
    mockParticipantQueries()
    apiMock.get.mockImplementation((url: string) => {
      if (url === '/courses/course-1') {
        return Promise.resolve({
          data: { ...course, capacity: 1, enrolled_count: 1 },
        })
      }
      if (url === '/courses/course-1/students') {
        return Promise.resolve({ data: { items: [enrolledStudent] } })
      }
      if (url.startsWith('/courses/course-1/available-students')) {
        return Promise.resolve({ data: { items: [availableStudent] } })
      }
      throw new Error(`Unexpected GET ${url}`)
    })
    apiMock.post.mockRejectedValue({
      code: 'COURSE_CAPACITY_REACHED',
      message: 'No quedan cupos',
    })
    renderParticipants()

    await user.click(
      await screen.findByRole('button', { name: 'Agregar estudiante' }),
    )
    const enrollButton = await screen.findByRole('button', { name: 'Inscribir' })
    expect(enrollButton).toBeEnabled()
    await user.click(enrollButton)

    expect(await screen.findByRole('alert')).toHaveTextContent('No quedan cupos')
  })

  it('lets an administrator view enrolled students without enrollment controls', async () => {
    mockParticipantQueries()
    renderParticipants('Admin')

    expect(await screen.findByText('Estudiante inscrito')).toBeInTheDocument()
    expect(
      screen.queryByRole('button', { name: 'Agregar estudiante' }),
    ).not.toBeInTheDocument()
    expect(
      screen.queryByRole('searchbox', {
        name: 'Buscar estudiantes disponibles',
      }),
    ).not.toBeInTheDocument()
    expect(screen.getByText(/no inscribir estudiantes/i)).toBeInTheDocument()
  })
})
