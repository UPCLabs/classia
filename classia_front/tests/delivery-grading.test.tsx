import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { AuthContext } from '../src/auth/context'
import DeliveryGradingView from '../src/views/StudentCourses/DeliveryGradingView'
import type { Entrega } from '../src/types/api'

const apiMock = vi.hoisted(() => ({
  get: vi.fn(),
  put: vi.fn(),
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

const delivery: Entrega = {
  id: 'delivery-1',
  activity_id: 'activity-1',
  student: {
    id: 'student-1',
    name: 'Ana Estudiante',
    email: 'ana@classia.test',
  },
  status: 'submitted',
  version: 2,
  submitted_at: '2026-11-02T21:14:00Z',
  updated_at: '2026-11-02T21:14:00Z',
  evaluation: null,
}

function renderGrading() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
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
        <MemoryRouter
          initialEntries={[
            '/activities/activity-1/deliveries/delivery-1/evaluation',
          ]}
        >
          <Routes>
            <Route
              path="/activities/:activityId/deliveries/:deliveryId/evaluation"
              element={<DeliveryGradingView />}
            />
            <Route
              path="/activities/:activityId"
              element={<p>Detalle de la actividad</p>}
            />
          </Routes>
        </MemoryRouter>
      </AuthContext.Provider>
    </QueryClientProvider>,
  )
}

function mockGrading(current: Entrega = delivery) {
  apiMock.get.mockImplementation((url: string) => {
    if (url === '/activities/activity-1') {
      return Promise.resolve({ data: activity })
    }
    if (url === '/activities/activity-1/deliveries') {
      return Promise.resolve({ data: { items: [current], total: 1 } })
    }
    throw new Error(`Unexpected GET ${url}`)
  })
}

beforeEach(() => {
  vi.clearAllMocks()
})

describe('DeliveryGradingView', () => {
  it('envía nota, retroalimentación y la versión vista', async () => {
    const user = userEvent.setup()
    mockGrading()
    apiMock.put.mockResolvedValue({
      data: { ...delivery, status: 'graded', version: 3 },
    })
    renderGrading()

    const score = await screen.findByLabelText('Nota (0,0 a 5,0)')
    await user.type(score, '4.5')
    await user.type(
      screen.getByLabelText(/Retroalimentación/),
      'Buen modelo',
    )
    await user.click(
      screen.getByRole('button', { name: 'Guardar calificación' }),
    )

    await waitFor(() => expect(apiMock.put).toHaveBeenCalledTimes(1))
    expect(apiMock.put).toHaveBeenCalledWith(
      '/deliveries/delivery-1/evaluation',
      {
        score: 4.5,
        feedback: 'Buen modelo',
        expected_version: 2,
      },
    )
    expect(await screen.findByText('Detalle de la actividad')).toBeInTheDocument()
  })

  it('envía feedback null cuando se deja vacío', async () => {
    const user = userEvent.setup()
    mockGrading()
    apiMock.put.mockResolvedValue({
      data: { ...delivery, status: 'graded', version: 3 },
    })
    renderGrading()

    await user.type(await screen.findByLabelText('Nota (0,0 a 5,0)'), '3')
    await user.click(
      screen.getByRole('button', { name: 'Guardar calificación' }),
    )

    await waitFor(() =>
      expect(apiMock.put).toHaveBeenCalledWith(
        '/deliveries/delivery-1/evaluation',
        { score: 3, feedback: null, expected_version: 2 },
      ),
    )
  })

  it('rechaza una nota fuera de 0,0 a 5,0', async () => {
    const user = userEvent.setup()
    mockGrading()
    renderGrading()

    const score = await screen.findByLabelText('Nota (0,0 a 5,0)')
    await user.clear(score)
    await user.type(score, '6')
    await user.click(
      screen.getByRole('button', { name: 'Guardar calificación' }),
    )

    expect(
      await screen.findByText(
        'La nota debe estar entre 0,0 y 5,0 con un solo decimal',
      ),
    ).toBeInTheDocument()
    expect(apiMock.put).not.toHaveBeenCalled()
  })

  it('muestra el 409, no guarda la nota y recarga la entrega nueva', async () => {
    const user = userEvent.setup()
    let listCalls = 0
    apiMock.get.mockImplementation((url: string) => {
      if (url === '/activities/activity-1') {
        return Promise.resolve({ data: activity })
      }
      if (url === '/activities/activity-1/deliveries') {
        listCalls += 1
        const current =
          listCalls > 1
            ? {
                ...delivery,
                status: 'replaced' as const,
                version: 3,
                updated_at: '2026-11-03T16:00:00Z',
              }
            : delivery
        return Promise.resolve({ data: { items: [current], total: 1 } })
      }
      throw new Error(`Unexpected GET ${url}`)
    })
    apiMock.put.mockRejectedValue({
      code: 'DELIVERY_VERSION_CONFLICT',
      message: 'La entrega cambió; recarga para ver la versión vigente',
    })
    renderGrading()

    await user.type(await screen.findByLabelText('Nota (0,0 a 5,0)'), '4')
    await user.click(
      screen.getByRole('button', { name: 'Guardar calificación' }),
    )

    expect(
      await screen.findByText(/La entrega cambió: el estudiante reemplazó/),
    ).toBeInTheDocument()
    expect(await screen.findByText('Reemplazada')).toBeInTheDocument()
    expect(listCalls).toBeGreaterThan(1)
  })
})
