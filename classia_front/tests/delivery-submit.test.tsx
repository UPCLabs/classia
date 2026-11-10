import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import { AuthContext } from '../src/auth/context'
import DeliveryFormView from '../src/views/StudentCourses/DeliveryFormView'
import type { Entrega } from '../src/types/api'

const apiMock = vi.hoisted(() => ({
  get: vi.fn(),
  post: vi.fn(),
  put: vi.fn(),
}))

vi.mock('../src/api/httpClient', () => ({ default: apiMock }))

const futureActivity = {
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
  version: 2,
  submitted_at: '2026-11-02T21:14:00Z',
  updated_at: '2026-11-03T15:02:00Z',
  evaluation: null,
}

function docxFile(name = 'entrega.docx') {
  return new File([new Uint8Array(1024)], name, {
    type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
  })
}

function renderDeliveryForm() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
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
          completeMfaSignIn: vi.fn(),
          signOut: vi.fn(),
        }}
      >
        <MemoryRouter initialEntries={['/activities/activity-1/delivery']}>
          <Routes>
            <Route
              path="/activities/:activityId/delivery"
              element={<DeliveryFormView />}
            />
            <Route
              path="/activities/:activityId/delivery/status"
              element={<p>Estado de la entrega</p>}
            />
          </Routes>
        </MemoryRouter>
      </AuthContext.Provider>
    </QueryClientProvider>,
  )
}

function mockActivityAndDelivery(
  activity = futureActivity,
  currentDelivery: Entrega | null = null,
) {
  apiMock.get.mockImplementation((url: string) => {
    if (url === '/activities/activity-1') {
      return Promise.resolve({ data: activity })
    }
    if (url === '/activities/activity-1/deliveries/me') {
      return Promise.resolve({ data: { delivery: currentDelivery } })
    }
    throw new Error(`Unexpected GET ${url}`)
  })
}

beforeEach(() => {
  vi.clearAllMocks()
  vi.restoreAllMocks()
})

describe('DeliveryFormView', () => {
  it('sube un Docx al endpoint de la actividad', async () => {
    const user = userEvent.setup()
    mockActivityAndDelivery()
    apiMock.post.mockResolvedValue({ data: delivery })
    const file = docxFile()
    renderDeliveryForm()

    await user.upload(await screen.findByLabelText(/Archivo/), file)
    await user.click(screen.getByRole('button', { name: 'Subir entrega' }))

    await waitFor(() => expect(apiMock.post).toHaveBeenCalledTimes(1))
    const [url, body] = apiMock.post.mock.calls[0]
    expect(url).toBe('/activities/activity-1/deliveries')
    expect(body).toBeInstanceOf(FormData)
    expect((body as FormData).get('file')).toBe(file)
  })

  it('rechaza un archivo que no es Docx antes de enviarlo', async () => {
    const user = userEvent.setup()
    mockActivityAndDelivery()
    renderDeliveryForm()

    const input = await screen.findByLabelText(/Archivo/)
    fireEvent.change(input, {
      target: { files: [docxFile('entrega.pdf')] },
    })
    await user.click(screen.getByRole('button', { name: 'Subir entrega' }))

    expect(
      await screen.findByText('El archivo debe estar en formato Docx'),
    ).toBeInTheDocument()
    expect(apiMock.post).not.toHaveBeenCalled()
  })

  it('bloquea la entrega cuando el plazo ya venció', async () => {
    mockActivityAndDelivery({
      ...futureActivity,
      due_at: '2020-01-01T00:00:00Z',
    })
    renderDeliveryForm()

    expect(
      await screen.findByText('El plazo de entrega ya venció.'),
    ).toBeInTheDocument()
    expect(screen.queryByLabelText(/Archivo/)).not.toBeInTheDocument()
    expect(
      screen.queryByRole('button', { name: 'Subir entrega' }),
    ).not.toBeInTheDocument()
  })

  it('avisa y envía la versión al reemplazar una entrega calificada', async () => {
    const user = userEvent.setup()
    const graded: Entrega = {
      ...delivery,
      status: 'graded',
      version: 3,
      evaluation: {
        score: 4.5,
        feedback: 'Buen trabajo',
        graded_at: '2026-11-03T15:02:00Z',
        updated_at: '2026-11-03T15:02:00Z',
      },
    }
    mockActivityAndDelivery(futureActivity, graded)
    apiMock.put.mockResolvedValue({ data: { ...graded, version: 4 } })
    const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true)
    const file = docxFile('reemplazo.docx')
    renderDeliveryForm()

    expect(
      await screen.findByText(
        'Si reemplazas esta entrega calificada, se borrará la nota.',
      ),
    ).toBeInTheDocument()

    await user.upload(screen.getByLabelText(/Archivo/), file)
    await user.click(
      screen.getByRole('button', { name: 'Reemplazar entrega' }),
    )

    expect(confirmSpy).toHaveBeenCalled()
    await waitFor(() => expect(apiMock.put).toHaveBeenCalledTimes(1))
    const [url, body] = apiMock.put.mock.calls[0]
    expect(url).toBe('/deliveries/delivery-1')
    expect((body as FormData).get('expected_version')).toBe('3')
  })

  it('muestra el 409 y recarga la entrega', async () => {
    const user = userEvent.setup()
    mockActivityAndDelivery(futureActivity, delivery)
    apiMock.put.mockRejectedValue({
      code: 'DELIVERY_VERSION_CONFLICT',
      message: 'La entrega cambió; recarga para ver la versión vigente',
    })
    renderDeliveryForm()

    await user.upload(await screen.findByLabelText(/Archivo/), docxFile())
    await user.click(
      screen.getByRole('button', { name: 'Reemplazar entrega' }),
    )

    expect(
      await screen.findByText(
        'La entrega cambió; recarga para ver la versión vigente',
      ),
    ).toBeInTheDocument()
    await waitFor(() =>
      expect(
        apiMock.get.mock.calls.filter(
          ([url]) => url === '/activities/activity-1/deliveries/me',
        ).length,
      ).toBeGreaterThan(1),
    )
  })
})
