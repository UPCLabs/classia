import { describe, expect, it } from 'vitest'
import {
  isActivityDue,
  isBogotaDateTimeInFuture,
} from '../src/api/activities'

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

describe('isActivityDue', () => {
  const now = new Date('2026-11-10T15:00:00Z').getTime()

  it('considera vencida una fecha límite anterior', () => {
    expect(isActivityDue('2026-11-10T14:59:00Z', now)).toBe(true)
  })

  it('considera vigente una fecha límite posterior', () => {
    expect(isActivityDue('2026-11-10T15:01:00Z', now)).toBe(false)
  })

  it('considera vencido el mismo instante', () => {
    expect(isActivityDue('2026-11-10T15:00:00Z', now)).toBe(true)
  })
})
