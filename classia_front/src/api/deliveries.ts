import httpClient from './httpClient'
import type {
  CalificacionInput,
  Entrega,
  EntregaPropia,
  EntregasRecibidas,
} from '../types/api'

export const MAX_DELIVERY_BYTES = 20 * 1024 * 1024

const DOCX_MIME =
  'application/vnd.openxmlformats-officedocument.wordprocessingml.document'

function fileFormData(file: File): FormData {
  const form = new FormData()
  form.append('file', file)
  return form
}

/**
 * Valida el archivo en el cliente con las mismas reglas del contrato (DEV-10).
 * Devuelve el mensaje de error o `null` si el archivo es aceptable.
 */
export function validateDeliveryFile(file: File): string | null {
  if (!file.name.toLowerCase().endsWith('.docx')) {
    return 'El archivo debe estar en formato Docx'
  }

  if (file.size > MAX_DELIVERY_BYTES) {
    return 'El archivo supera el tamaño máximo de 20 MB'
  }

  return null
}

export async function submitDelivery(
  activityId: string,
  file: File,
): Promise<Entrega> {
  const { data } = await httpClient.post<Entrega>(
    `/activities/${activityId}/deliveries`,
    fileFormData(file),
  )
  return data
}

export async function replaceDelivery(
  deliveryId: string,
  file: File,
  expectedVersion: number,
): Promise<Entrega> {
  const form = fileFormData(file)
  form.append('expected_version', String(expectedVersion))

  const { data } = await httpClient.put<Entrega>(
    `/deliveries/${deliveryId}`,
    form,
  )
  return data
}

export async function getMyDelivery(activityId: string): Promise<Entrega | null> {
  const { data } = await httpClient.get<EntregaPropia>(
    `/activities/${activityId}/deliveries/me`,
  )
  return data.delivery
}

export async function listReceivedDeliveries(
  activityId: string,
): Promise<EntregasRecibidas> {
  const { data } = await httpClient.get<EntregasRecibidas>(
    `/activities/${activityId}/deliveries`,
  )
  return data
}

export async function downloadDeliveryFile(deliveryId: string): Promise<Blob> {
  const { data } = await httpClient.get<Blob>(
    `/deliveries/${deliveryId}/file`,
    { responseType: 'blob' },
  )
  return data
}

export async function gradeDelivery(
  deliveryId: string,
  input: CalificacionInput,
): Promise<Entrega> {
  const { data } = await httpClient.put<Entrega>(
    `/deliveries/${deliveryId}/evaluation`,
    input,
  )
  return data
}

export { DOCX_MIME }
