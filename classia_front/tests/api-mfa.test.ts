import { beforeEach, describe, expect, it, vi } from 'vitest'
import { challengeMfa, resetMfa, setupMfa } from '../src/api/mfa'

const apiMock = vi.hoisted(() => ({
  post: vi.fn(),
}))

vi.mock('../src/api/httpClient', () => ({ default: apiMock }))

beforeEach(() => {
  vi.clearAllMocks()
})

describe('setupMfa', () => {
  it('devuelve el secreto y la URI para el código QR', async () => {
    apiMock.post.mockResolvedValue({
      data: {
        secret: 'JBSWY3DPEHPK3PXP',
        url: 'otpauth://totp/Classia:estudiante@classia.test?secret=JBSWY3DPEHPK3PXP',
      },
    })

    await expect(setupMfa()).resolves.toMatchObject({
      secret: 'JBSWY3DPEHPK3PXP',
    })
    expect(apiMock.post).toHaveBeenCalledWith('/auth/setup-mfa')
  })
})

describe('challengeMfa', () => {
  it('devuelve true cuando el código es válido', async () => {
    apiMock.post.mockResolvedValue({ data: { verified: true } })

    await expect(challengeMfa('123456')).resolves.toBe(true)
    expect(apiMock.post).toHaveBeenCalledWith('/auth/challenge-mfa', {
      code: '123456',
    })
  })

  it('devuelve false cuando el código no es válido', async () => {
    apiMock.post.mockResolvedValue({ data: { verified: false } })

    await expect(challengeMfa('000000')).resolves.toBe(false)
  })

  it('propaga el 400 con su código', async () => {
    apiMock.post.mockRejectedValue({
      code: 'BAD_REQUEST',
      message: 'MFA is not set',
    })

    await expect(challengeMfa('123456')).rejects.toMatchObject({
      code: 'BAD_REQUEST',
    })
  })
})

describe('resetMfa', () => {
  it('envía la cuenta y el motivo obligatorio', async () => {
    apiMock.post.mockResolvedValue({ data: undefined })

    await resetMfa('user-1', 'Perdió el teléfono')

    expect(apiMock.post).toHaveBeenCalledWith('/users/user-1/reset-mfa', {
      reason: 'Perdió el teléfono',
    })
  })
})
