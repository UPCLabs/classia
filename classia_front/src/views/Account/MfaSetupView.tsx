import { useState, type FormEvent } from 'react'
import { useMutation } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { QRCodeSVG } from 'qrcode.react'
import { challengeMfa, setupMfa } from '../../api/mfa'
import getErrorMessage from '../../auth/getErrorMessage'
import type { MfaSetup } from '../../types/api'

export default function MfaSetupView() {
  const [setup, setSetup] = useState<MfaSetup | null>(null)
  const [activated, setActivated] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const setupMutation = useMutation({
    mutationFn: setupMfa,
    onSuccess: (data) => {
      setSetup(data)
      setError(null)
    },
    onError: (requestError) => setError(getErrorMessage(requestError)),
  })

  const verifyMutation = useMutation({
    mutationFn: challengeMfa,
    onSuccess: (verified) => {
      if (verified) {
        setActivated(true)
        setError(null)
      } else {
        setError('Código inválido o vencido. Inténtalo de nuevo.')
      }
    },
    onError: (requestError) => setError(getErrorMessage(requestError)),
  })

  function handleVerify(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const data = new FormData(event.currentTarget)
    const code = String(data.get('code') ?? '').trim()

    if (!/^\d{6}$/.test(code)) {
      setError('El código debe tener 6 dígitos.')
      return
    }

    setError(null)
    verifyMutation.mutate(code)
  }

  return (
    <main className="mx-auto max-w-3xl px-4 py-10">
      <Link className="text-verde-medio underline" to="/account/change-password">
        Volver a ajustes de cuenta
      </Link>

      <section className="mt-5 rounded-2xl bg-white p-6 shadow-sm sm:p-8">
        <p className="text-sm font-semibold uppercase tracking-wider text-verde-medio">
          Seguridad
        </p>
        <h1 className="mt-2 text-3xl font-bold text-verde-oscuro">
          Configurar MFA
        </h1>
        <p className="mt-3 text-verde-oscuro/75">
          Protege tu cuenta con un código temporal de seis dígitos, además de tu
          contraseña.
        </p>

        {activated ? (
          <div className="mt-6">
            <p className="rounded-lg bg-crema p-4 font-semibold text-verde-oscuro" role="status">
              MFA activado correctamente.
            </p>
            <Link
              className="mt-4 inline-block rounded-lg bg-dorado px-5 py-2 font-semibold text-verde-oscuro"
              to="/dashboard"
            >
              Volver al inicio
            </Link>
          </div>
        ) : !setup ? (
          <div className="mt-6">
            {error && (
              <p className="mb-4 rounded-lg bg-crema p-3 text-verde-oscuro" role="alert">
                {error}
              </p>
            )}
            <button
              className="rounded-lg bg-dorado px-5 py-2 font-semibold text-verde-oscuro transition hover:brightness-110 disabled:opacity-60"
              disabled={setupMutation.isPending}
              onClick={() => setupMutation.mutate()}
              type="button"
            >
              {setupMutation.isPending
                ? 'Generando...'
                : 'Comenzar configuración'}
            </button>
          </div>
        ) : (
          <div className="mt-6 grid gap-6">
            <div>
              <h2 className="text-xl font-semibold text-verde-oscuro">
                1. Escanea el código QR
              </h2>
              <p className="mt-1 text-sm text-verde-oscuro/75">
                Usa Google Authenticator, Microsoft Authenticator u otra
                aplicación TOTP.
              </p>
              <div
                aria-label="Código QR para configurar MFA"
                className="mt-4 w-fit rounded-xl bg-white p-4 shadow-sm"
                role="img"
              >
                <QRCodeSVG size={192} value={setup.url} />
              </div>
              <details className="mt-3 text-sm text-verde-oscuro/75">
                <summary className="cursor-pointer font-semibold">
                  No puedo escanear el código
                </summary>
                <p className="mt-2">
                  Ingresa esta clave manualmente:
                </p>
                <code className="mt-1 block break-all rounded-lg bg-crema px-3 py-2 text-verde-oscuro">
                  {setup.secret}
                </code>
              </details>
            </div>

            <form className="grid gap-3" noValidate onSubmit={handleVerify}>
              <h2 className="text-xl font-semibold text-verde-oscuro">
                2. Confirma el código
              </h2>
              <div className="grid gap-1">
                <label className="font-medium" htmlFor="mfa-code">
                  Código de 6 dígitos
                </label>
                <input
                  autoComplete="one-time-code"
                  className="w-40 rounded-lg border border-verde-medio/30 bg-crema px-3 py-2 text-center text-lg tracking-[0.3em]"
                  id="mfa-code"
                  inputMode="numeric"
                  maxLength={6}
                  name="code"
                  pattern="[0-9]{6}"
                />
              </div>

              {error && (
                <p className="rounded-lg bg-crema p-3 text-verde-oscuro" role="alert">
                  {error}
                </p>
              )}

              <button
                className="justify-self-start rounded-lg bg-dorado px-5 py-2 font-semibold text-verde-oscuro transition hover:brightness-110 disabled:opacity-60"
                disabled={verifyMutation.isPending}
                type="submit"
              >
                {verifyMutation.isPending ? 'Verificando...' : 'Activar MFA'}
              </button>
            </form>
          </div>
        )}
      </section>
    </main>
  )
}
