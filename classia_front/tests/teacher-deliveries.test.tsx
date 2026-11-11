import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { AuthContext } from '../src/auth/context'
import ActivityDetailView from '../src/views/StudentCourses/ActivityDetailView'
import type { Entrega } from '../src/types/api'

const apiMock = vi.hoisted(() => ({
  get: vi.fn(),
}))

vi.mock('../src/api/httpClient', () => ({ default: apiMock }))

const activity = {
  id: 'activity-1',
  course_id: 'course-1',
  title: 'Taller de pruebas',
  description: 'Entrega el taller en formato Docx.',
  due_at: '2026-12-01T04:59:00Z',
  status: 'published' as const,
  published_at: '2026-11-01T10:00:00Z',
  created_at: '2026-11-01T09:00:00Z',
  updated_at: '2026-11-01T09:00:00Z',
}

function delivery(overrides: Partial<Entrega> = {}): Entrega {
  return {
    id: 'delivery-1',
    activity_id: 'activity-1',
    student: {
      id: 'student-1',
      name: 'Ana Estudiante',
      email: 'ana@classia.test',
    },
    status: 'submitted',
    version: 1,
    submitted_at: '2026-11-02T21:14:00Z',
    updated_at: '2026-11-02T21:14:00Z',
    evaluation: null,
    ...overrides,
  }
}

function renderActivity() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <AuthContext.Provider
        value={{
          user: {
            id: 'teacher-1',
            name: 'Docente Prueba',
            email: 'docente@classia.test',
            role: 'Teacher',
          },
          isLoading: false,
          error: null,
          signIn: vi.fn(),
          completeMfaSignIn: vi.fn(),
          signOut: vi.fn(),
        }}
      >
        <MemoryRouter initialEntries={['/activities/activity-1']}>
          <Routes>
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

function mockDeliveries(items: Entrega[], total = items.length) {
  apiMock.get.mockImplementation((url: string) => {
    if (url === '/activities/activity-1') {
      return Promise.resolve({ data: activity })
    }
    if (url === '/activities/activity-1/deliveries') {
      return Promise.resolve({ data: { items, total } })
    }
    if (url === '/deliveries/delivery-1/file') {
      return Promise.resolve({ data: new Blob(['docx']) })
    }
    throw new Error(`Unexpected GET ${url}`)
  })
}

beforeEach(() => {
  vi.clearAllMocks()
  vi.restoreAllMocks()
})

describe('ActivityDetailView — entregas recibidas', () => {
  it('lista las entregas con estudiante, estado, fecha y total', async () => {
    mockDeliveries([
      delivery(),
      delivery({
        id: 'delivery-2',
        student: {
          id: 'student-2',
          name: 'Luis Estudiante',
          email: 'luis@classia.test',
        },
        status: 'graded',
        version: 2,
        evaluation: {
          score: 4.5,
          feedback: 'Buen trabajo',
          graded_at: '2026-11-03T15:02:00Z',
          updated_at: '2026-11-03T15:02:00Z',
        },
      }),
    ])
    renderActivity()

    expect(await screen.findByText('Ana Estudiante')).toBeInTheDocument()
    expect(screen.getByText('Luis Estudiante')).toBeInTheDocument()
    expect(screen.getByText('2 entregas')).toBeInTheDocument()
    expect(screen.getByText('Recibida')).toBeInTheDocument()
    expect(screen.getByText('Calificada')).toBeInTheDocument()
    expect(screen.getByText('4.5 / 5.0')).toBeInTheDocument()

    expect(screen.getByRole('link', { name: 'Calificar' })).toHaveAttribute(
      'href',
      '/activities/activity-1/deliveries/delivery-1/evaluation',
    )
    expect(
      screen.getByRole('link', { name: 'Corregir calificación' }),
    ).toHaveAttribute(
      'href',
      '/activities/activity-1/deliveries/delivery-2/evaluation',
    )
  })

  it('muestra el mensaje cuando no hay entregas', async () => {
    mockDeliveries([], 0)
    renderActivity()

    expect(
      await screen.findByText('Aún no hay entregas para esta actividad'),
    ).toBeInTheDocument()
    expect(screen.getByText('0 entregas')).toBeInTheDocument()
  })

  it('descarga el archivo de la entrega', async () => {
    const user = userEvent.setup()
    mockDeliveries([delivery()])
    const createObjectURL = vi.fn(() => 'blob:entrega')
    const revokeObjectURL = vi.fn()
    vi.stubGlobal('URL', {
      ...URL,
      createObjectURL,
      revokeObjectURL,
    })
    const click = vi
      .spyOn(HTMLAnchorElement.prototype, 'click')
      .mockImplementation(() => {})
    renderActivity()

    await user.click(await screen.findByRole('button', { name: 'Descargar' }))

    await waitFor(() =>
      expect(apiMock.get).toHaveBeenCalledWith('/deliveries/delivery-1/file', {
        responseType: 'blob',
      }),
    )
    expect(createObjectURL).toHaveBeenCalled()
    expect(click).toHaveBeenCalled()
    expect(revokeObjectURL).toHaveBeenCalledWith('blob:entrega')
    vi.unstubAllGlobals()
  })
})
