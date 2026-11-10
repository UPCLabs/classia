import { useState, type FormEvent } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { getActivity, isActivityDue } from '../../api/activities'
import {
  getMyDelivery,
  replaceDelivery,
  submitDelivery,
  validateDeliveryFile,
} from '../../api/deliveries'
import getErrorMessage from '../../auth/getErrorMessage'
import {
  deliveryStatusLabel,
  formatDeliveryDate,
} from './deliveryPresentation'

const bogotaDateTimeFormatter = new Intl.DateTimeFormat('es-CO', {
  dateStyle: 'long',
  timeStyle: 'short',
  timeZone: 'America/Bogota',
})

export default function DeliveryFormView() {
  const { activityId } = useParams()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [file, setFile] = useState<File | null>(null)
  const [formError, setFormError] = useState<string | null>(null)

  const activityQuery = useQuery({
    queryKey: ['activity', activityId],
    enabled: Boolean(activityId),
    queryFn: () => {
      if (!activityId) throw new Error('La actividad no es válida.')
      return getActivity(activityId)
    },
  })

  const deliveryQuery = useQuery({
    queryKey: ['delivery', activityId],
    enabled: Boolean(activityId),
    queryFn: () => {
      if (!activityId) throw new Error('La actividad no es válida.')
      return getMyDelivery(activityId)
    },
  })

  const mutation = useMutation({
    mutationFn: async (selectedFile: File) => {
      if (!activityId) throw new Error('La actividad no es válida.')

      const current = deliveryQuery.data
      if (current) {
        return replaceDelivery(current.id, selectedFile, current.version)
      }
      return submitDelivery(activityId, selectedFile)
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['delivery', activityId] })
      navigate(`/activities/${activityId}/delivery/status`, {
        state: { message: 'Tu entrega se guardó correctamente.' },
      })
    },
    onError: (error) => {
      setFormError(getErrorMessage(error))
      if (errorCode(error) === 'DELIVERY_VERSION_CONFLICT') {
        void queryClient.invalidateQueries({
          queryKey: ['delivery', activityId],
        })
      }
    },
  })

  if (activityQuery.isLoading || deliveryQuery.isLoading) {
    return <p className="p-10 text-center" role="status">Cargando entrega...</p>
  }

  if (activityQuery.error || deliveryQuery.error) {
    return (
      <main className="mx-auto max-w-3xl px-4 py-10">
        <p className="rounded-lg bg-crema p-4 text-verde-oscuro" role="alert">
          {getErrorMessage(activityQuery.error ?? deliveryQuery.error)}
        </p>
        <Link className="mt-4 inline-block text-verde-medio underline" to="/courses">
          Volver a cursos
        </Link>
      </main>
    )
  }

  const activity = activityQuery.data
  if (!activity || !activityId) return null

  if (activity.status !== 'published') {
    return (
      <main className="mx-auto max-w-3xl px-4 py-10">
        <p className="rounded-lg bg-crema p-4 text-verde-oscuro" role="alert">
          Actividad no encontrada.
        </p>
        <Link
          className="mt-4 inline-block text-verde-medio underline"
          to={`/courses/${activity.course_id}`}
        >
          Volver al curso
        </Link>
      </main>
    )
  }

  const delivery = deliveryQuery.data ?? null
  const isPastDue = isActivityDue(activity.due_at)
  const isReplacing = Boolean(delivery)

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setFormError(null)

    if (!file) {
      setFormError('Selecciona un archivo Docx.')
      return
    }

    const validationError = validateDeliveryFile(file)
    if (validationError) {
      setFormError(validationError)
      return
    }

    if (
      delivery?.status === 'graded' &&
      !window.confirm(
        'Esta entrega ya está calificada. Si la reemplazas, se borrará la nota. ¿Continuar?',
      )
    ) {
      return
    }

    mutation.mutate(file)
  }

  return (
    <main className="mx-auto max-w-3xl px-4 py-10">
      <Link
        className="text-verde-medio underline"
        to={`/activities/${activityId}`}
      >
        Volver a la actividad
      </Link>

      <article className="mt-5 rounded-2xl border-l-4 border-dorado bg-white p-6 shadow-sm sm:p-8">
        <h1 className="text-3xl font-bold text-verde-oscuro">
          {isReplacing ? 'Reemplazar entrega' : 'Entregar actividad'}
        </h1>
        <h2 className="mt-2 text-xl font-semibold text-verde-oscuro">
          {activity.title}
        </h2>
        <p className="mt-4 whitespace-pre-wrap text-verde-oscuro/85">
          {activity.description}
        </p>

        <dl className="mt-6 border-t border-verde-medio/25 pt-4">
          <dt className="text-sm text-verde-oscuro/65">Fecha límite</dt>
          <dd className="mt-1 font-semibold text-verde-oscuro">
            {bogotaDateTimeFormatter.format(new Date(activity.due_at))} (Bogotá)
          </dd>
        </dl>
      </article>

      <section className="mt-6 rounded-2xl bg-white p-6 shadow-sm sm:p-8">
        {delivery && (
          <div className="mb-6 rounded-xl bg-crema p-4">
            <p className="font-semibold text-verde-oscuro">
              Entrega actual: {deliveryStatusLabel(delivery.status)}
            </p>
            <p className="mt-1 text-sm text-verde-oscuro/75">
              Última actualización: {formatDeliveryDate(delivery.updated_at)}
            </p>
            {delivery.status === 'graded' && (
              <p className="mt-3 text-sm font-semibold text-verde-oscuro" role="alert">
                Si reemplazas esta entrega calificada, se borrará la nota.
              </p>
            )}
          </div>
        )}

        {isPastDue ? (
          <div>
            <p className="rounded-lg bg-crema p-4 font-semibold text-verde-oscuro" role="alert">
              El plazo de entrega ya venció.
            </p>
            <Link
              className="mt-4 inline-block text-verde-medio underline"
              to={`/activities/${activityId}/delivery/status`}
            >
              Ver el estado de mi entrega
            </Link>
          </div>
        ) : (
          <form className="grid gap-4" onSubmit={handleSubmit}>
            <div className="grid gap-1">
              <label className="font-medium" htmlFor="delivery-file">
                Archivo (Docx, máximo 20 MB)
              </label>
              <input
                accept=".docx,application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                className="w-full max-w-md rounded-lg border border-verde-medio/30 bg-crema px-3 py-2"
                id="delivery-file"
                onChange={(event) => setFile(event.target.files?.[0] ?? null)}
                type="file"
              />
              <p className="text-sm text-verde-oscuro/70">
                Solo se acepta un archivo .docx de hasta 20 MB.
              </p>
            </div>

            {formError && (
              <p className="rounded-lg bg-crema p-3 text-verde-oscuro" role="alert">
                {formError}
              </p>
            )}

            <button
              className="justify-self-start rounded-lg bg-dorado px-6 py-3 font-semibold text-verde-oscuro transition hover:brightness-110 disabled:opacity-60"
              disabled={mutation.isPending}
              type="submit"
            >
              {mutation.isPending
                ? 'Guardando...'
                : isReplacing
                  ? 'Reemplazar entrega'
                  : 'Subir entrega'}
            </button>
          </form>
        )}
      </section>
    </main>
  )
}

function errorCode(error: unknown): string | null {
  if (typeof error === 'object' && error !== null && 'code' in error) {
    const code = (error as { code?: unknown }).code
    return typeof code === 'string' ? code : null
  }
  return null
}
