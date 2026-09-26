import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, vi } from 'vitest'
import type { ReactNode } from 'react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { AuthContext } from '../src/auth/context'
import CourseCreateView from '../src/views/EstudentCourses/CourseCreateView'
import CourseDetailView from '../src/views/EstudentCourses/CourseDetailView'
import CourseEditView from '../src/views/EstudentCourses/CourseEditView'

const apiMock = vi.hoisted(() => ({
  get: vi.fn(),
  post: vi.fn(),
  patch: vi.fn(),
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
  enrolled_count: 7,
  created_at: '2026-09-25T10:00:00Z',
  updated_at: '2026-09-25T10:00:00Z',
}

function renderCourseView(
  element: ReactNode,
  role: 'Teacher' | 'Admin' | 'Student',
  initialPath: string,
) {
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
        <MemoryRouter initialEntries={[initialPath]}>
          <Routes>
            <Route path="/courses/new" element={element} />
            <Route path="/courses/:courseId" element={element} />
            <Route path="/courses/:courseId/edit" element={element} />
          </Routes>
        </MemoryRouter>
      </AuthContext.Provider>
    </QueryClientProvider>,
  )
}

beforeEach(() => {
  vi.clearAllMocks()
})

describe('CourseCreateView', () => {
  it('submits course data with the authenticated teacher as owner', async () => {
    const user = userEvent.setup()
    apiMock.post.mockResolvedValue({ data: { id: 'course-1' } })
    renderCourseView(<CourseCreateView />, 'Teacher', '/courses/new')

    await user.type(screen.getByLabelText('Nombre'), 'Programación')
    await user.type(screen.getByLabelText('Código'), 'PRG101')
    await user.type(screen.getByLabelText('Cupo'), '25')
    await user.click(screen.getByRole('button', { name: 'Crear curso' }))

    expect(apiMock.post).toHaveBeenCalledWith('/courses', {
      name: 'Programación',
      code: 'PRG101',
      teacher_id: 'teacher-1',
      status: 'active',
      capacity: 25,
    })
  })
})

describe('CourseEditView', () => {
  it('loads an owned course and patches only changed fields', async () => {
    const user = userEvent.setup()
    apiMock.get.mockImplementation((url: string) =>
      Promise.resolve({
        data:
          url === '/courses/course-1'
            ? course
            : { items: [{ id: 'teacher-1', name: 'Docente de prueba', email: course.teacher.email, role: 'Teacher', status: 'active' }] },
      }),
    )
    apiMock.patch.mockResolvedValue({
      data: { ...course, name: 'Programación actualizada' },
    })
    renderCourseView(
      <CourseEditView />,
      'Teacher',
      '/courses/course-1/edit',
    )

    const name = await screen.findByLabelText('Nombre')
    await user.clear(name)
    await user.type(name, 'Programación actualizada')
    await user.click(screen.getByRole('button', { name: 'Guardar cambios' }))

    expect(apiMock.patch).toHaveBeenCalledWith('/courses/course-1', {
      name: 'Programación actualizada',
    })
  })
})

describe('CourseDetailView', () => {
  it('shows edit only to the teacher responsible for the course', async () => {
    apiMock.get.mockResolvedValue({ data: course })
    renderCourseView(
      <CourseDetailView />,
      'Teacher',
      '/courses/course-1',
    )

    expect(await screen.findByText('Programación')).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Editar curso' })).toHaveAttribute(
      'href',
      '/courses/course-1/edit',
    )
  })

  it('hides edit from a student', async () => {
    apiMock.get.mockResolvedValue({ data: course })
    renderCourseView(
      <CourseDetailView />,
      'Student',
      '/courses/course-1',
    )

    expect(await screen.findByText('Programación')).toBeInTheDocument()
    expect(screen.queryByRole('link', { name: 'Editar curso' })).not.toBeInTheDocument()
  })
})
