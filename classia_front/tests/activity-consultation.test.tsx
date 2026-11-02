import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, vi } from 'vitest'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { AuthContext } from '../src/auth/context'
import ActivityDetailView from '../src/views/StudentCourses/ActivityDetailView'
import CourseDetailView from '../src/views/StudentCourses/CourseDetailView'

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
  enrolled_count: 7,
  created_at: '2026-09-25T10:00:00Z',
  updated_at: '2026-09-25T10:00:00Z',
}

const draftActivity = {
  id: 'activity-draft',
  course_id: 'course-1',
  title: 'Actividad en borrador',
  description: 'Instrucciones para el borrador.',
  due_at: '2099-12-01T04:59:00Z',
  status: 'draft' as const,
  published_at: null,
  created_at: '2099-10-25T15:20:00Z',
  updated_at: '2099-10-25T15:20:00Z',
}

const publishedActivity = {
  ...draftActivity,
  id: 'activity-published',
  title: 'Actividad publicada',
  description: 'Instrucciones para estudiantes.',
  status: 'published' as const,
  published_at: '2099-10-26T15:20:00Z',
}

type Role = 'Teacher' | 'Student' | 'Admin' | 'SuperAdmin'

function renderActivityPage(path: string, role: Role) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <AuthContext.Provider
        value={{
          user: {
            id: role === 'Teacher' ? 'teacher-1' : 'user-1',
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
        <MemoryRouter initialEntries={[path]}>
          <Routes>
            <Route path="/courses/:courseId" element={<CourseDetailView />} />
            <Route
              path="/activities/:activityId"
              element={<ActivityDetailView />}
            />
          </Routes>
        </MemoryRouter>
      </AuthContext.Provider>
    </QueryClientProvider>,
  )
}

beforeEach(() => {
  vi.clearAllMocks()
  vi.stubGlobal('confirm', vi.fn(() => true))
})

describe('ActivityList', () => {
  it('shows draft and published activities with their status to the teacher', async () => {
    apiMock.get.mockImplementation((url: string) => {
      if (url === '/courses/course-1') {
        return Promise.resolve({ data: course })
      }
      if (url === '/courses/course-1/activities') {
        return Promise.resolve({
          data: { items: [draftActivity, publishedActivity] },
        })
      }
      throw new Error(`Unexpected GET ${url}`)
    })
    renderActivityPage('/courses/course-1', 'Teacher')

    expect(await screen.findByText('Actividad en borrador')).toBeInTheDocument()
    expect(screen.getByText('Actividad publicada')).toBeInTheDocument()
    expect(screen.getByText('Borrador')).toBeInTheDocument()
    expect(screen.getByText('Publicada')).toBeInTheDocument()
    expect(
      screen.getByRole('link', { name: /Actividad en borrador/ }),
    ).toHaveAttribute('href', '/activities/activity-draft')
    expect(screen.getAllByRole('link', { name: 'Editar actividad' })).toEqual([
      expect.objectContaining({
        href: expect.stringContaining('/activities/activity-draft/edit'),
      }),
      expect.objectContaining({
        href: expect.stringContaining('/activities/activity-published/edit'),
      }),
    ])
  })

  it('only displays published activities to students', async () => {
    apiMock.get.mockImplementation((url: string) => {
      if (url === '/courses/course-1') {
        return Promise.resolve({ data: course })
      }
      if (url === '/courses/course-1/activities') {
        return Promise.resolve({
          data: { items: [draftActivity, publishedActivity] },
        })
      }
      throw new Error(`Unexpected GET ${url}`)
    })
    renderActivityPage('/courses/course-1', 'Student')

    expect(await screen.findByText('Actividad publicada')).toBeInTheDocument()
    expect(screen.queryByText('Actividad en borrador')).not.toBeInTheDocument()
    expect(screen.queryByText('Borrador')).not.toBeInTheDocument()
    expect(
      screen.queryByRole('link', { name: 'Editar actividad' }),
    ).not.toBeInTheDocument()
    expect(apiMock.get).toHaveBeenCalledWith('/courses/course-1/activities')
  })

  it('lets the responsible teacher confirm and publish a draft activity', async () => {
    let published = false
    apiMock.get.mockImplementation((url: string) => {
      if (url === '/courses/course-1') {
        return Promise.resolve({ data: course })
      }
      if (url === '/courses/course-1/activities') {
        return Promise.resolve({
          data: {
            items: [
              published
                ? { ...draftActivity, status: 'published', published_at: '2099-10-26T15:20:00Z' }
                : draftActivity,
            ],
          },
        })
      }
      throw new Error(`Unexpected GET ${url}`)
    })
    apiMock.post.mockImplementation(() => {
      published = true
      return Promise.resolve({
        data: {
          ...draftActivity,
          status: 'published',
          published_at: '2099-10-26T15:20:00Z',
        },
      })
    })
    renderActivityPage('/courses/course-1', 'Teacher')

    const publishButton = await screen.findByRole('button', { name: 'Publicar' })
    await userEvent.click(publishButton)

    expect(window.confirm).toHaveBeenCalledWith(
      '¿Publicar "Actividad en borrador" para los estudiantes?',
    )
    expect(apiMock.post).toHaveBeenCalledWith(
      '/activities/activity-draft/publish',
    )
    expect(await screen.findByText('Actividad publicada.')).toBeInTheDocument()
    expect(await screen.findByText('Publicada')).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Publicar' })).not.toBeInTheDocument()
  })

  it('does not show publish controls to admins', async () => {
    apiMock.get.mockImplementation((url: string) =>
      Promise.resolve({
        data:
          url === '/courses/course-1'
            ? course
            : { items: [draftActivity] },
      }),
    )
    renderActivityPage('/courses/course-1', 'Admin')

    expect(await screen.findByText('Actividad en borrador')).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Publicar' })).not.toBeInTheDocument()
    expect(
      screen.queryByRole('link', { name: 'Editar actividad' }),
    ).not.toBeInTheDocument()
  })

  it('shows the API error when publishing is rejected', async () => {
    apiMock.get.mockImplementation((url: string) =>
      Promise.resolve({
        data:
          url === '/courses/course-1'
            ? course
            : { items: [draftActivity] },
      }),
    )
    apiMock.post.mockRejectedValue({
      code: 'ACTIVITY_ALREADY_PUBLISHED',
      message: 'La actividad ya está publicada',
    })
    renderActivityPage('/courses/course-1', 'Teacher')

    await userEvent.click(
      await screen.findByRole('button', { name: 'Publicar' }),
    )

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'La actividad ya está publicada',
    )
  })

  it('shows the empty-list message when the course has no activities', async () => {
    apiMock.get.mockImplementation((url: string) =>
      Promise.resolve({
        data: url === '/courses/course-1' ? course : { items: [] },
      }),
    )
    renderActivityPage('/courses/course-1', 'Student')

    expect(
      await screen.findByText('No hay actividades para mostrar.'),
    ).toBeInTheDocument()
  })
})

describe('ActivityDetailView', () => {
  it('shows the activity details and status to the teacher', async () => {
    apiMock.get.mockResolvedValue({ data: draftActivity })
    renderActivityPage('/activities/activity-draft', 'Teacher')

    expect(
      await screen.findByRole('heading', { name: 'Actividad en borrador' }),
    ).toBeInTheDocument()
    expect(screen.getByText('Instrucciones para el borrador.')).toBeInTheDocument()
    expect(screen.getByText('Borrador')).toBeInTheDocument()
    expect(apiMock.get).toHaveBeenCalledWith('/activities/activity-draft')
  })

  it('does not expose a draft activity to a student', async () => {
    apiMock.get.mockResolvedValue({ data: draftActivity })
    renderActivityPage('/activities/activity-draft', 'Student')

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Actividad no encontrada.',
    )
    expect(
      screen.queryByRole('heading', { name: 'Actividad en borrador' }),
    ).not.toBeInTheDocument()
    expect(
      screen.queryByText('Instrucciones para el borrador.'),
    ).not.toBeInTheDocument()
  })
})
