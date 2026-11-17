import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { MemoryRouter, Route, Routes } from 'react-router-dom'
import MfaSetupView from '../src/views/Account/MfaSetupView'

const apiMock = vi.hoisted(() => ({
  post: vi.fn(),
}))

vi.mock('../src/api/httpClient', () => ({ default: apiMock }))

const setupResponse = {
  secret: 'JBSWY3DPEHPK3PXP',
  url: 'otpauth://totp/Classia:ana@example.com?secret=JBSWY3DPEHPK3PXP&issuer=Classia',
}

function renderMfaSetup() {
  const queryClient = new QueryClient({
    defaultOptions: { mutations: { retry: false } },
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={['/account/mfa']}>
        <Routes>
          <Route path="/account/mfa" element={<MfaSetupView />} />
          <Route path="/dashboard" element={<p>Inicio</p>} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

beforeEach(() => {
  vi.clearAllMocks()
})

describe('MfaSetupView', () => {
  it('genera el QR y activa el MFA con un código válido', async () => {
    const user = userEvent.setup()
    apiMock.post.mockImplementation((url: string) => {
      if (url === '/auth/setup-mfa') {
        return Promise.resolve({ data: setupResponse })
      }
      if (url === '/auth/challenge-mfa') {
        return Promise.resolve({ data: { verified: true } })
      }
      throw new Error(`Unexpected POST ${url}`)
    })
    renderMfaSetup()

    await user.click(
      screen.getByRole('button', { name: 'Comenzar configuración' }),
    )

    expect(
      await screen.findByRole('img', {
        name: 'Código QR para configurar MFA',
      }),
    ).toBeInTheDocument()
    expect(screen.getByText(setupResponse.secret)).toBeInTheDocument()

    await user.type(screen.getByLabelText('Código de 6 dígitos'), '123456')
    await user.click(screen.getByRole('button', { name: 'Activar MFA' }))

    await waitFor(() =>
      expect(apiMock.post).toHaveBeenCalledWith('/auth/challenge-mfa', {
        code: '123456',
      }),
    )
    expect(
      await screen.findByText('MFA activado correctamente.'),
    ).toBeInTheDocument()
  })

  it('muestra un mensaje claro cuando el código es inválido o venció', async () => {
    const user = userEvent.setup()
    apiMock.post.mockImplementation((url: string) => {
      if (url === '/auth/setup-mfa') {
        return Promise.resolve({ data: setupResponse })
      }
      if (url === '/auth/challenge-mfa') {
        return Promise.resolve({ data: { verified: false } })
      }
      throw new Error(`Unexpected POST ${url}`)
    })
    renderMfaSetup()

    await user.click(
      screen.getByRole('button', { name: 'Comenzar configuración' }),
    )
    await user.type(
      await screen.findByLabelText('Código de 6 dígitos'),
      '000000',
    )
    await user.click(screen.getByRole('button', { name: 'Activar MFA' }))

    expect(
      await screen.findByText(
        'Código inválido o vencido. Inténtalo de nuevo.',
      ),
    ).toBeInTheDocument()
  })

  it('exige seis dígitos antes de llamar al servidor', async () => {
    const user = userEvent.setup()
    apiMock.post.mockResolvedValue({ data: setupResponse })
    renderMfaSetup()

    await user.click(
      screen.getByRole('button', { name: 'Comenzar configuración' }),
    )
    await user.type(
      await screen.findByLabelText('Código de 6 dígitos'),
      '12345',
    )
    await user.click(screen.getByRole('button', { name: 'Activar MFA' }))

    expect(
      await screen.findByText('El código debe tener 6 dígitos.'),
    ).toBeInTheDocument()
    expect(
      apiMock.post.mock.calls.some(
        ([url]) => url === '/auth/challenge-mfa',
      ),
    ).toBe(false)
  })
})
