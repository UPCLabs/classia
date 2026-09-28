import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, vi } from 'vitest'
import type { ReactNode } from 'react'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { AuthContext } from '../src/auth/context'
import CourseCreateView from '../src/views/StudentCourses/CourseCreateView'
import CourseDetailView from '../src/views/StudentCourses/CourseDetailView'
import CourseEditView from '../src/views/StudentCourses/CourseEditView'
import CoursesView from '../src/views/StudentCourses/CoursesView'
import DashboardView from '../src/views/DashboardView'

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
  role: 'Teacher' | 'Admin' | 'SuperAdmin' | 'Student',
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
            <Route path="/courses" element={<CoursesView />} />
            <Route path="/courses/:courseId" element={<CourseDetailView />} />
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
  it('creates with the current API contract and returns to the refreshed list', async () => {
    const user = userEvent.setup()
    apiMock.get.mockImplementation((url: string) =>
      Promise.resolve({
        data: url === '/courses' ? { items: [course] } : course,
      }),
    )
    apiMock.post.mockResolvedValue({ data: course })
    renderCourseView(<CourseCreateView />, 'Teacher', '/courses/new')

    await user.type(await screen.findByLabelText('Nombre'), 'Programación')
    await user.type(screen.getByLabelText('Código'), 'PRG101')
    await user.clear(screen.getByLabelText('Cupo'))
    await user.type(screen.getByLabelText('Cupo'), '25')
    await user.click(screen.getByRole('button', { name: 'Crear curso' }))

    expect(apiMock.post).toHaveBeenCalledWith('/courses', {
      name: 'Programación',
      code: 'PRG101',
      teacher_id: 'teacher-1',
      status: 'active',
      capacity: 25,
    })
    expect(
      await screen.findByText('El curso Programación se creó correctamente.'),
    ).toHaveAttribute('role', 'status')
    expect(await screen.findByRole('link', { name: /Programación/ })).toHaveAttribute(
      'href',
      '/courses/course-1',
    )
    expect(apiMock.get).toHaveBeenCalledWith('/courses')
    await user.click(screen.getByRole('link', { name: /Programación/ }))
    expect(
      await screen.findByRole('heading', { name: 'Programación' }),
    ).toBeInTheDocument()
  })

  it('lets an administrator assign an active teacher', async () => {
    const user = userEvent.setup()
    const teacher = {
      id: 'teacher-2',
      name: 'Otra docente',
      email: 'otra@example.com',
      role: 'Teacher' as const,
      status: 'active' as const,
      created_at: '',
      updated_at: '',
    }
    apiMock.get.mockImplementation((url: string) =>
      Promise.resolve({
        data:
          url === '/users?role=Teacher&status=active'
            ? { items: [teacher] }
            : { items: [course] },
      }),
    )
    apiMock.post.mockResolvedValue({ data: course })
    renderCourseView(<CourseCreateView />, 'Admin', '/courses/new')

    await user.type(await screen.findByLabelText('Nombre'), 'Programación')
    await user.type(screen.getByLabelText('Código'), 'PRG101')
    await user.selectOptions(screen.getByLabelText('Docente'), 'teacher-2')
    await user.clear(screen.getByLabelText('Cupo'))
    await user.type(screen.getByLabelText('Cupo'), '25')
    await user.click(screen.getByRole('button', { name: 'Crear curso' }))

    expect(apiMock.post).toHaveBeenCalledWith('/courses', {
      name: 'Programación',
      code: 'PRG101',
      teacher_id: 'teacher-2',
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

  it('cancels editing and returns to details without submitting changes', async () => {
    const user = userEvent.setup()
    apiMock.get.mockResolvedValue({ data: course })
    renderCourseView(
      <CourseEditView />,
      'Teacher',
      '/courses/course-1/edit',
    )

    const name = await screen.findByLabelText('Nombre')
    await user.clear(name)
    await user.type(name, 'Cambio descartado')
    await user.click(screen.getByRole('link', { name: 'Cancelar' }))

    expect(
      await screen.findByRole('heading', { name: 'Programación' }),
    ).toBeInTheDocument()
    expect(apiMock.patch).not.toHaveBeenCalled()
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
    expect(screen.getByRole('link', { name: 'Participantes' })).toHaveAttribute(
      'href',
      '/courses/course-1/students',
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
    expect(screen.queryByRole('link', { name: 'Participantes' })).not.toBeInTheDocument()
  })

  it('allows an administrator to edit a course', async () => {
    apiMock.get.mockResolvedValue({ data: course })
    renderCourseView(
      <CourseDetailView />,
      'Admin',
      '/courses/course-1',
    )

    expect(await screen.findByRole('link', { name: 'Editar curso' })).toHaveAttribute(
      'href',
      '/courses/course-1/edit',
    )
    expect(screen.getByRole('link', { name: 'Participantes' })).toHaveAttribute(
      'href',
      '/courses/course-1/students',
    )
  })

  it('allows a superadministrator to edit a course', async () => {
    apiMock.get.mockResolvedValue({ data: course })
    renderCourseView(
      <CourseDetailView />,
      'SuperAdmin',
      '/courses/course-1',
    )

    expect(await screen.findByRole('link', { name: 'Editar curso' })).toHaveAttribute(
      'href',
      '/courses/course-1/edit',
    )
  })
})

describe('DashboardView', () => {
  it('shows administrators a courses link', () => {
    render(
      <AuthContext.Provider
        value={{
          user: {
            id: 'admin-1',
            name: 'Admin',
            email: 'admin@example.com',
            role: 'Admin',
          },
          isLoading: false,
          error: null,
          signIn: vi.fn(),
          signOut: vi.fn(),
        }}
      >
        <MemoryRouter>
          <DashboardView />
        </MemoryRouter>
      </AuthContext.Provider>,
    )

    expect(screen.getByRole('link', { name: /Cursos/ })).toHaveAttribute(
      'href',
      '/courses',
    )
  })
})
