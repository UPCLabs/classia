import httpClient from './httpClient'
import type { MfaChallenge, MfaSetup } from '../types/api'

/**
 * El contrato de MFA (UC-005 y UC-006) todavía no está publicado: estas rutas
 * siguen el backend en progreso (`sm/add-mfa`). Cuando David publique el
 * contrato en el Issue de DEV-10, este módulo es el único que debe cambiar.
 */
export async function setupMfa(): Promise<MfaSetup> {
  const { data } = await httpClient.post<MfaSetup>('/auth/setup-mfa')
  return data
}

export async function challengeMfa(code: string): Promise<boolean> {
  const { data } = await httpClient.post<MfaChallenge>('/auth/challenge-mfa', {
    code,
  })
  return data.verified
}

/** Restablece el MFA de otra cuenta; `reason` es obligatorio. */
export async function resetMfa(userId: string, reason: string): Promise<void> {
  await httpClient.post(`/users/${userId}/reset-mfa`, { reason })
}
