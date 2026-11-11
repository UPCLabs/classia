import type { DeliveryStatus } from '../../types/api'

export const deliveryStatusLabels: Record<DeliveryStatus, string> = {
  submitted: 'Recibida',
  replaced: 'Reemplazada',
  graded: 'Calificada',
}

export function deliveryStatusLabel(status: DeliveryStatus | null): string {
  return status ? deliveryStatusLabels[status] : 'Sin entrega'
}

const bogotaDateTimeFormatter = new Intl.DateTimeFormat('es-CO', {
  dateStyle: 'long',
  timeStyle: 'short',
  timeZone: 'America/Bogota',
})

export function formatDeliveryDate(value: string): string {
  return `${bogotaDateTimeFormatter.format(new Date(value))} (Bogotá)`
}

export function formatScore(score: number): string {
  return score.toFixed(1)
}
