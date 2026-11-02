import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, vi } from 'vitest'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { AuthContext } from '../src/auth/context'
import ActivityCreateView from '../src/views/StudentCourses/ActivityCreateView'
import ActivityEditView from '../src/views/StudentCourses/ActivityEditView'

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

const activity = {
  id: 'activity-1',
  course_id: 'course-1',
  title: 'Taller inicial',
  description: 'Resolver el taller.',
  due_at: '2099-12-01T04:59:30.000Z',
  status: 'draft' as const,
  published_at: null,
  created_at: '2099-10-25T15:20:00Z',
  updated_at: '2099-10-25T15:20:00Z',
}

function renderActivityView(initialPath: string, teacherId = 'teacher-1') {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <AuthContext.Provider
        value={{
          user: {
            id: teacherId,
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
        <MemoryRouter initialEntries={[initialPath]}>
          <Routes>
            <Route
              path="/courses/:courseId/activities/new"
              element={<ActivityCreateView />}
            />
            <Route
              path="/activities/:activityId/edit"
              element={<ActivityEditView />}
            />
            <Route path="/courses/:courseId" element={<p>Detalle del curso</p>} />
          </Routes>
        </MemoryRouter>
      </AuthContext.Provider>
    </QueryClientProvider>,
  )
}

beforeEach(() => {
  vi.clearAllMocks()
})

describe('ActivityCreateView', () => {
  it('creates an activity and converts the Bogotá deadline to UTC', async () => {
    const user = userEvent.setup()
    apiMock.get.mockResolvedValue({ data: course })
    apiMock.post.mockResolvedValue({
      data: { ...activity, title: 'Taller inicial' },
    })
    renderActivityView('/courses/course-1/activities/new')

    await user.type(await screen.findByLabelText('Título'), 'Taller inicial')
    await user.type(
      screen.getByLabelText('Descripción'),
      'Resolver el taller.',
    )
    fireEvent.change(screen.getByLabelText('Fecha y hora límite (Bogotá)'), {
      target: { value: '2099-11-30T23:59' },
    })
    await user.click(screen.getByRole('button', { name: 'Crear actividad' }))

    expect(apiMock.post).toHaveBeenCalledWith(
      '/courses/course-1/activities',
      {
        title: 'Taller inicial',
        description: 'Resolver el taller.',
        due_at: '2099-12-01T04:59:00.000Z',
      },
    )
    expect(await screen.findByText('Detalle del curso')).toBeInTheDocument()
  })

  it('does not show the form to a teacher who does not own the course', async () => {
    apiMock.get.mockResolvedValue({
      data: { ...course, teacher: { ...course.teacher, id: 'teacher-2' } },
    })
    renderActivityView('/courses/course-1/activities/new')

    expect(
      await screen.findByText(
        'Solo el docente responsable puede crear actividades en este curso.',
      ),
    ).toBeInTheDocument()
    expect(screen.queryByLabelText('Título')).not.toBeInTheDocument()
    expect(apiMock.post).not.toHaveBeenCalled()
  })
})

describe('ActivityEditView', () => {
  it('patches only changed fields while preserving the original deadline', async () => {
    const user = userEvent.setup()
    apiMock.get.mockImplementation((url: string) => {
      if (url === '/activities/activity-1') {
        return Promise.resolve({ data: activity })
      }
      if (url === '/courses/course-1') {
        return Promise.resolve({ data: course })
      }
      throw new Error(`Unexpected GET ${url}`)
    })
    apiMock.patch.mockResolvedValue({
      data: { ...activity, title: 'Taller actualizado' },
    })
    renderActivityView('/activities/activity-1/edit')

    const title = await screen.findByLabelText('Título')
    expect(screen.getByLabelText('Fecha y hora límite (Bogotá)')).toHaveValue(
      '2099-11-30T23:59',
    )
    await user.clear(title)
    await user.type(title, 'Taller actualizado')
    await user.click(
      screen.getByRole('button', { name: 'Guardar cambios' }),
    )

    expect(apiMock.patch).toHaveBeenCalledWith('/activities/activity-1', {
      title: 'Taller actualizado',
    })
    expect(await screen.findByText('Detalle del curso')).toBeInTheDocument()
  })
})
