import { useState } from 'react'
import { useForm, useWatch } from 'react-hook-form'
import { useNavigate } from 'react-router-dom'
import httpClient from '../../api/httpClient'
import { useAuth } from '../../auth/context'
import getErrorMessage from '../../auth/getErrorMessage'

type ChangePasswordForm = {
  newPassword: string
  confirmationPassword: string
}

export default function ChangePasswordView() {
  const { signOut } = useAuth()
  const navigate = useNavigate()
  const [submitError, setSubmitError] = useState<string | null>(null)
  const {
    control,
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<ChangePasswordForm>()
  const newPassword = useWatch({ control, name: 'newPassword' })

  async function onSubmit(data: ChangePasswordForm) {
    setSubmitError(null)

    try {
      await httpClient.patch<void>('/users/change-password', {
        new_password: data.newPassword,
      })
      await signOut()
      navigate('/auth/login', { replace: true })
    } catch (error: unknown) {
      setSubmitError(getErrorMessage(error))
    }
  }

  return (
    <main className="flex min-h-screen items-center justify-center bg-verde-oscuro px-4">
      <div className="w-full max-w-sm rounded-2xl bg-verde-medio p-8 shadow-xl">
        <h1 className="mb-6 text-center text-2xl font-bold text-crema">
          Cambiar contraseña
        </h1>

        <form
          className="flex flex-col gap-4"
          onSubmit={handleSubmit(onSubmit)}
        >
          <div className="flex flex-col gap-1">
            <label className="text-sm font-medium text-crema" htmlFor="new-password">
              Nueva contraseña
            </label>
            <input
              autoComplete="new-password"
              className="rounded-lg bg-crema px-3 py-2 text-verde-oscuro outline-none focus:ring-2 focus:ring-dorado"
              id="new-password"
              {...register('newPassword', {
                required: 'Este campo es obligatorio',
                minLength: {
                  value: 8,
                  message: 'La contraseña debe tener al menos 8 caracteres',
                },
                maxLength: {
                  value: 20,
                  message: 'La contraseña no puede superar 20 caracteres',
                },
              })}
              type="password"
            />
            {errors.newPassword && (
              <span className="text-xs text-red-200" role="alert">
                {errors.newPassword.message}
              </span>
            )}
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-sm font-medium text-crema" htmlFor="confirmation-password">
              Confirmar contraseña
            </label>
            <input
              autoComplete="new-password"
              className="rounded-lg bg-crema px-3 py-2 text-verde-oscuro outline-none focus:ring-2 focus:ring-dorado"
              id="confirmation-password"
              {...register('confirmationPassword', {
                required: 'Este campo es obligatorio',
                validate: (value) =>
                  value === newPassword || 'Las contraseñas no coinciden',
              })}
              type="password"
            />
            {errors.confirmationPassword && (
              <span className="text-xs text-red-200" role="alert">
                {errors.confirmationPassword.message}
              </span>
            )}
          </div>

          {submitError && (
            <p className="rounded-lg bg-crema px-3 py-2 text-sm text-red-800" role="alert">
              {submitError}
            </p>
          )}

          <button
            className="mt-2 rounded-lg bg-dorado py-2 font-semibold text-verde-oscuro transition hover:brightness-110 disabled:cursor-wait disabled:opacity-70"
            disabled={isSubmitting}
            type="submit"
          >
            {isSubmitting ? 'Guardando...' : 'Guardar cambios'}
          </button>
        </form>
      </div>
    </main>
  )
}