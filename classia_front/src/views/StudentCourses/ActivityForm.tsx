import { useForm } from 'react-hook-form'
import {
  bogotaDateTimeToUtc,
  utcToBogotaDateTime,
} from '../../api/activities'
import type { Activity, ActivityCreate } from '../../types/api'

export type ActivityFormValues = {
  title: string
  description: string
  due_at: string
}

type ActivityFormProps = {
  initialActivity?: Activity
  isSubmitting: boolean
  submitError: string | null
  submitLabel: string
  onSubmit: (values: ActivityCreate) => Promise<void>
}

export default function ActivityForm({
  initialActivity,
  isSubmitting,
  submitError,
  submitLabel,
  onSubmit,
}: ActivityFormProps) {
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<ActivityFormValues>({
    defaultValues: {
      title: initialActivity?.title ?? '',
      description: initialActivity?.description ?? '',
      due_at: initialActivity
        ? utcToBogotaDateTime(initialActivity.due_at)
        : '',
    },
  })

  async function submit(values: ActivityFormValues) {
    await onSubmit({
      title: values.title.trim(),
      description: values.description,
      due_at: bogotaDateTimeToUtc(values.due_at),
    })
  }

  return (
    <form
      className="grid gap-6 rounded-2xl bg-crema/40 p-5 shadow-sm sm:p-8"
      onSubmit={handleSubmit(submit)}
    >
      <fieldset className="grid gap-5 rounded-xl border border-verde-medio/25 p-4 sm:p-6">
        <legend className="px-2 text-lg font-semibold text-verde-oscuro">
          General
        </legend>
        <p className="-mt-3 text-sm text-verde-oscuro/70">
          Define el título y las instrucciones de la actividad.
        </p>

        <div className="grid gap-1">
          <label className="font-medium" htmlFor="activity-title">
            Título <span className="text-sm font-normal text-verde-oscuro/70">(obligatorio)</span>
          </label>
          <input
            className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2"
            id="activity-title"
            maxLength={150}
            aria-invalid={Boolean(errors.title)}
            {...register('title', {
              required: 'El título es obligatorio',
              validate: (value) =>
                value.trim().length > 0 || 'El título es obligatorio',
              maxLength: {
                value: 150,
                message: 'El título no puede superar 150 caracteres',
              },
            })}
          />
          {errors.title && (
            <p className="text-sm text-verde-oscuro" role="alert">
              {errors.title.message}
            </p>
          )}
        </div>

        <div className="grid gap-1">
          <label className="font-medium" htmlFor="activity-description">
            Descripción <span className="text-sm font-normal text-verde-oscuro/70">(obligatoria)</span>
          </label>
          <textarea
            className="min-h-36 rounded-lg border border-verde-medio/30 bg-crema px-3 py-2"
            id="activity-description"
            maxLength={5000}
            aria-invalid={Boolean(errors.description)}
            {...register('description', {
              required: 'La descripción es obligatoria',
              validate: (value) =>
                value.trim().length > 0 || 'La descripción es obligatoria',
              maxLength: {
                value: 5000,
                message: 'La descripción no puede superar 5000 caracteres',
              },
            })}
          />
          {errors.description && (
            <p className="text-sm text-verde-oscuro" role="alert">
              {errors.description.message}
            </p>
          )}
          <p className="text-sm text-verde-oscuro/70">
            Escribe las instrucciones que verá el estudiante.
          </p>
        </div>
      </fieldset>

      <fieldset className="grid gap-4 rounded-xl border border-verde-medio/25 p-4 sm:p-6">
        <legend className="px-2 text-lg font-semibold text-verde-oscuro">
          Disponibilidad
        </legend>
        <p className="-mt-3 text-sm text-verde-oscuro/70">
          Establece hasta cuándo estará disponible esta actividad.
        </p>

        <div className="grid gap-1">
          <label className="font-medium" htmlFor="activity-due-at">
            Fecha y hora límite <span className="text-sm font-normal text-verde-oscuro/70">(obligatoria)</span>
          </label>
          <input
            className="w-full max-w-sm rounded-lg border border-verde-medio/30 bg-crema px-3 py-2"
            id="activity-due-at"
            step="60"
            type="datetime-local"
            aria-invalid={Boolean(errors.due_at)}
            {...register('due_at', {
              required: 'La fecha límite es obligatoria',
              validate: (value) =>
                (Boolean(value) &&
                  new Date(bogotaDateTimeToUtc(value)).getTime() > Date.now()) ||
                'La fecha límite debe ser futura',
            })}
          />
          {errors.due_at && (
            <p className="text-sm text-verde-oscuro" role="alert">
              {errors.due_at.message}
            </p>
          )}
          <p className="text-sm text-verde-oscuro/70">
            Hora de Bogotá (UTC−5). Se enviará al servidor en UTC.
          </p>
        </div>
      </fieldset>

      {!initialActivity && (
        <p className="rounded-lg bg-crema px-4 py-3 text-sm text-verde-oscuro">
          La actividad se creará como borrador.
        </p>
      )}

      {submitError && (
        <p className="rounded-lg bg-crema p-3 text-verde-oscuro" role="alert">
          {submitError}
        </p>
      )}

      <button
        className="justify-self-start rounded-lg bg-dorado px-6 py-3 font-semibold text-verde-oscuro transition hover:brightness-110 disabled:opacity-60"
        disabled={isSubmitting}
        type="submit"
      >
        {isSubmitting ? 'Guardando...' : submitLabel}
      </button>
    </form>
  )
}
