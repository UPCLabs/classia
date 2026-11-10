import { describe, expect, it } from 'vitest'
import { isBogotaDateTimeInFuture } from '../src/api/activities'

describe('isBogotaDateTimeInFuture', () => {
  const now = new Date('2026-11-10T15:00:00Z').getTime()

  it('acepta una fecha de Bogotá posterior al momento actual', () => {
    expect(isBogotaDateTimeInFuture('2026-11-10T11:00', now)).toBe(true)
  })

  it('rechaza una fecha de Bogotá ya vencida', () => {
    expect(isBogotaDateTimeInFuture('2026-11-10T09:00', now)).toBe(false)
  })

  it('rechaza el mismo instante', () => {
    expect(isBogotaDateTimeInFuture('2026-11-10T10:00', now)).toBe(false)
  })
})
