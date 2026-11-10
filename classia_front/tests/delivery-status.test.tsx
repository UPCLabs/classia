import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { AuthContext } from '../src/auth/context'
import DeliveryStatusView from '../src/views/StudentCourses/DeliveryStatusView'
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
  due_at: '2099-12-01T04:59:00Z',
  status: 'published' as const,
  published_at: '2026-11-01T10:00:00Z',
  created_at: '2026-11-01T09:00:00Z',
  updated_at: '2026-11-01T09:00:00Z',
}

const delivery: Entrega = {
  id: 'delivery-1',
  activity_id: 'activity-1',
  student: {
    id: 'student-1',
    name: 'Estudiante Prueba',
    email: 'estudiante@classia.test',
  },
  status: 'submitted',
  version: 1,
  submitted_at: '2026-11-02T21:14:00Z',
  updated_at: '2026-11-02T21:14:00Z',
  evaluation: null,
}

function renderStatus() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <AuthContext.Provider
        value={{
          user: {
            id: 'student-1',
            name: 'Estudiante Prueba',
            email: 'estudiante@classia.test',
            role: 'Student',
          },
          isLoading: false,
          error: null,
          signIn: vi.fn(),
          signOut: vi.fn(),
        }}
      >
        <MemoryRouter
          initialEntries={['/activities/activity-1/delivery/status']}
        >
          <Routes>
            <Route
              path="/activities/:activityId/delivery/status"
              element={<DeliveryStatusView />}
            />
          </Routes>
        </MemoryRouter>
      </AuthContext.Provider>
    </QueryClientProvider>,
  )
}

function mockResponse(
  currentDelivery: Entrega | null,
  currentActivity = activity,
) {
  apiMock.get.mockImplementation((url: string) => {
    if (url === '/activities/activity-1') {
      return Promise.resolve({ data: currentActivity })
    }
    if (url === '/activities/activity-1/deliveries/me') {
      return Promise.resolve({ data: { delivery: currentDelivery } })
    }
    throw new Error(`Unexpected GET ${url}`)
  })
}

beforeEach(() => {
  vi.clearAllMocks()
})

describe('DeliveryStatusView', () => {
  it('muestra "Sin entrega" cuando el estudiante no ha entregado', async () => {
    mockResponse(null)
    renderStatus()

    expect(await screen.findByText('Sin entrega')).toBeInTheDocument()
    expect(
      screen.getByRole('link', { name: 'Subir entrega' }),
    ).toHaveAttribute('href', '/activities/activity-1/delivery')
  })

  it('muestra el estado recibida y que aún no hay calificación', async () => {
    mockResponse(delivery)
    renderStatus()

    expect(await screen.findByText('Recibida')).toBeInTheDocument()
    expect(
      screen.getByText('La entrega aún no tiene calificación.'),
    ).toBeInTheDocument()
  })

  it('muestra reemplazada con su versión', async () => {
    mockResponse({ ...delivery, status: 'replaced', version: 2 })
    renderStatus()

    expect(await screen.findByText('Reemplazada')).toBeInTheDocument()
    expect(screen.getByText('Versión 2')).toBeInTheDocument()
  })

  it('muestra la nota y la retroalimentación de una entrega calificada', async () => {
    mockResponse({
      ...delivery,
      status: 'graded',
      version: 3,
      evaluation: {
        score: 4.5,
        feedback: 'Buen modelo; revisa la cardinalidad.',
        graded_at: '2026-11-03T15:02:00Z',
        updated_at: '2026-11-03T15:02:00Z',
      },
    })
    renderStatus()

    expect(await screen.findByText('Calificada')).toBeInTheDocument()
    expect(screen.getByText('4.5 / 5.0')).toBeInTheDocument()
    expect(
      screen.getByText('Buen modelo; revisa la cardinalidad.'),
    ).toBeInTheDocument()
  })

  it('no ofrece subir entrega cuando el plazo venció', async () => {
    mockResponse(null, { ...activity, due_at: '2020-01-01T00:00:00Z' })
    renderStatus()

    expect(
      await screen.findByText('El plazo de entrega ya venció.'),
    ).toBeInTheDocument()
    expect(
      screen.queryByRole('link', { name: 'Subir entrega' }),
    ).not.toBeInTheDocument()
  })
})
