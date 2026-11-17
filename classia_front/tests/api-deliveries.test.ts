import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  downloadDeliveryFile,
  gradeDelivery,
  getMyDelivery,
  listReceivedDeliveries,
  MAX_DELIVERY_BYTES,
  replaceDelivery,
  submitDelivery,
  validateDeliveryFile,
} from '../src/api/deliveries'
import type { Entrega } from '../src/types/api'

const apiMock = vi.hoisted(() => ({
  get: vi.fn(),
  post: vi.fn(),
  put: vi.fn(),
}))

vi.mock('../src/api/httpClient', () => ({ default: apiMock }))

const entrega: Entrega = {
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

function docxFile(name = 'entrega.docx', size = 1024) {
  return new File([new Uint8Array(size)], name, {
    type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
  })
}

beforeEach(() => {
  vi.clearAllMocks()
})

describe('validateDeliveryFile', () => {
  it('acepta un Docx dentro del límite', () => {
    expect(validateDeliveryFile(docxFile())).toBeNull()
  })

  it('rechaza un archivo que no es Docx', () => {
    expect(validateDeliveryFile(docxFile('entrega.pdf'))).toBe(
      'El archivo debe estar en formato Docx',
    )
  })

  it('rechaza un archivo de más de 20 MB', () => {
    expect(validateDeliveryFile(docxFile('grande.docx', MAX_DELIVERY_BYTES + 1))).toBe(
      'El archivo supera el tamaño máximo de 20 MB',
    )
  })
})

describe('submitDelivery', () => {
  it('envía el archivo en multipart al endpoint de la actividad', async () => {
    apiMock.post.mockResolvedValue({ data: entrega })
    const file = docxFile()

    await expect(submitDelivery('activity-1', file)).resolves.toEqual(entrega)

    expect(apiMock.post).toHaveBeenCalledTimes(1)
    const [url, body] = apiMock.post.mock.calls[0]
    expect(url).toBe('/activities/activity-1/deliveries')
    expect(body).toBeInstanceOf(FormData)
    expect((body as FormData).get('file')).toBe(file)
  })
})

describe('replaceDelivery', () => {
  it('envía el archivo y la versión que vio el estudiante', async () => {
    apiMock.put.mockResolvedValue({ data: { ...entrega, version: 2 } })
    const file = docxFile()

    await replaceDelivery('delivery-1', file, 1)

    const [url, body] = apiMock.put.mock.calls[0]
    expect(url).toBe('/deliveries/delivery-1')
    expect(body).toBeInstanceOf(FormData)
    expect((body as FormData).get('file')).toBe(file)
    expect((body as FormData).get('expected_version')).toBe('1')
  })

  it('propaga el 409 de versión sin perder el código', async () => {
    apiMock.put.mockRejectedValue({
      code: 'DELIVERY_VERSION_CONFLICT',
      message: 'La entrega cambió; recarga para ver la versión vigente',
    })

    await expect(replaceDelivery('delivery-1', docxFile(), 1)).rejects.toMatchObject({
      code: 'DELIVERY_VERSION_CONFLICT',
    })
  })
})

describe('consultas de entregas', () => {
  it('devuelve null cuando el estudiante no ha entregado', async () => {
    apiMock.get.mockResolvedValue({ data: { delivery: null } })

    await expect(getMyDelivery('activity-1')).resolves.toBeNull()
    expect(apiMock.get).toHaveBeenCalledWith(
      '/activities/activity-1/deliveries/me',
    )
  })

  it('devuelve la entrega con su evaluación', async () => {
    const graded: Entrega = {
      ...entrega,
      status: 'graded',
      version: 2,
      evaluation: {
        score: 4.5,
        feedback: 'Buen trabajo',
        graded_at: '2026-11-03T15:02:00Z',
        updated_at: '2026-11-03T15:02:00Z',
      },
    }
    apiMock.get.mockResolvedValue({ data: { delivery: graded } })

    await expect(getMyDelivery('activity-1')).resolves.toEqual(graded)
  })

  it('lista las entregas recibidas con su total', async () => {
    apiMock.get.mockResolvedValue({
      data: { items: [entrega], total: 1 },
    })

    await expect(listReceivedDeliveries('activity-1')).resolves.toEqual({
      items: [entrega],
      total: 1,
    })
  })

  it('descarga el archivo como blob', async () => {
    const blob = new Blob(['docx'])
    apiMock.get.mockResolvedValue({ data: blob })

    await expect(downloadDeliveryFile('delivery-1')).resolves.toBe(blob)
    expect(apiMock.get).toHaveBeenCalledWith('/deliveries/delivery-1/file', {
      responseType: 'blob',
    })
  })
})

describe('gradeDelivery', () => {
  it('envía nota, retroalimentación y versión esperada', async () => {
    apiMock.put.mockResolvedValue({
      data: { ...entrega, status: 'graded', version: 2 },
    })

    await gradeDelivery('delivery-1', {
      score: 4.5,
      feedback: 'Buen modelo',
      expected_version: 1,
    })

    expect(apiMock.put).toHaveBeenCalledWith(
      '/deliveries/delivery-1/evaluation',
      { score: 4.5, feedback: 'Buen modelo', expected_version: 1 },
    )
  })

  it('propaga el 400 de validación con su código', async () => {
    apiMock.put.mockRejectedValue({
      code: 'VALIDATION_ERROR',
      message: 'La nota debe estar entre 0,0 y 5,0',
    })

    await expect(
      gradeDelivery('delivery-1', { score: 6, expected_version: 1 }),
    ).rejects.toMatchObject({ code: 'VALIDATION_ERROR' })
  })
})
