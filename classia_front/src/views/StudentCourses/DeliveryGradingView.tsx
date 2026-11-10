import { useState, type FormEvent } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { getActivity } from '../../api/activities'
import {
  downloadDeliveryFile,
  gradeDelivery,
  listReceivedDeliveries,
} from '../../api/deliveries'
import getErrorMessage from '../../auth/getErrorMessage'
import { saveBlob } from '../../shared/downloadBlob'
import type { CalificacionInput } from '../../types/api'
import {
  deliveryStatusLabel,
  formatDeliveryDate,
  formatScore,
} from './deliveryPresentation'

export default function DeliveryGradingView() {
  const { activityId, deliveryId } = useParams()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [formError, setFormError] = useState<string | null>(null)
  const [conflict, setConflict] = useState(false)
  const [downloadError, setDownloadError] = useState<string | null>(null)

  const activityQuery = useQuery({
    queryKey: ['activity', activityId],
    enabled: Boolean(activityId),
    queryFn: () => {
      if (!activityId) throw new Error('La actividad no es válida.')
      return getActivity(activityId)
    },
  })

  const deliveriesQuery = useQuery({
    queryKey: ['deliveries', activityId],
    enabled: Boolean(activityId),
    queryFn: () => {
      if (!activityId) throw new Error('La actividad no es válida.')
      return listReceivedDeliveries(activityId)
    },
  })

  const mutation = useMutation({
    mutationFn: (input: CalificacionInput) => {
      if (!deliveryId) throw new Error('La entrega no es válida.')
      return gradeDelivery(deliveryId, input)
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ['deliveries', activityId] })
      navigate(`/activities/${activityId}`, {
        state: { message: 'La calificación se guardó correctamente.' },
      })
    },
    onError: (error) => {
      setFormError(getErrorMessage(error))
      if (errorCode(error) === 'DELIVERY_VERSION_CONFLICT') {
        setConflict(true)
        void queryClient.invalidateQueries({
          queryKey: ['deliveries', activityId],
        })
      }
    },
  })

  if (activityQuery.isLoading || deliveriesQuery.isLoading) {
    return <p className="p-10 text-center" role="status">Cargando entrega...</p>
  }

  if (activityQuery.error || deliveriesQuery.error) {
    return (
      <main className="mx-auto max-w-3xl px-4 py-10">
        <p className="rounded-lg bg-crema p-4 text-verde-oscuro" role="alert">
          {getErrorMessage(activityQuery.error ?? deliveriesQuery.error)}
        </p>
        <Link className="mt-4 inline-block text-verde-medio underline" to="/courses">
          Volver a cursos
        </Link>
      </main>
    )
  }

  const activity = activityQuery.data
  const delivery =
    deliveriesQuery.data?.items.find((item) => item.id === deliveryId) ?? null

  if (!activity || !activityId) return null

  if (!delivery) {
    return (
      <main className="mx-auto max-w-3xl px-4 py-10">
        <p className="rounded-lg bg-crema p-4 text-verde-oscuro" role="alert">
          Entrega no encontrada.
        </p>
        <Link
          className="mt-4 inline-block text-verde-medio underline"
          to={`/activities/${activityId}`}
        >
          Volver a la actividad
        </Link>
      </main>
    )
  }

  async function handleDownload() {
    if (!delivery) return
    setDownloadError(null)
    try {
      const blob = await downloadDeliveryFile(delivery.id)
      saveBlob(blob, `entrega-${delivery.id}.docx`)
    } catch (error) {
      setDownloadError(getErrorMessage(error))
    }
  }

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (!delivery) return

    const data = new FormData(event.currentTarget)
    const scoreText = String(data.get('score') ?? '').trim()
    const score = Number(scoreText)

    if (
      !scoreText ||
      Number.isNaN(score) ||
      score < 0 ||
      score > 5 ||
      !/^\d(\.\d)?$/.test(scoreText)
    ) {
      setFormError('La nota debe estar entre 0,0 y 5,0 con un solo decimal')
      return
    }

    const feedbackText = String(data.get('feedback') ?? '').trim()
    setFormError(null)
    setConflict(false)

    mutation.mutate({
      score,
      feedback: feedbackText.length > 0 ? feedbackText : null,
      expected_version: delivery.version,
    })
  }

  return (
    <main className="mx-auto max-w-3xl px-4 py-10">
      <Link
        className="text-verde-medio underline"
        to={`/activities/${activityId}`}
      >
        Volver a la actividad
      </Link>

      <section className="mt-5 rounded-2xl bg-white p-6 shadow-sm sm:p-8">
        <p className="text-sm font-semibold uppercase tracking-wider text-verde-medio">
          Calificar entrega
        </p>
        <h1 className="mt-2 text-3xl font-bold text-verde-oscuro">
          {activity.title}
        </h1>

        <div className="mt-5 rounded-xl bg-crema p-4">
          <p className="font-semibold text-verde-oscuro">
            {delivery.student.name}
          </p>
          <p className="text-sm text-verde-oscuro/70">
            {delivery.student.email}
          </p>
          <dl className="mt-3 grid gap-2 text-sm sm:grid-cols-2">
            <div>
              <dt className="text-verde-oscuro/65">Estado</dt>
              <dd className="font-semibold text-verde-oscuro">
                {deliveryStatusLabel(delivery.status)}
              </dd>
            </div>
            <div>
              <dt className="text-verde-oscuro/65">Versión</dt>
              <dd className="font-semibold text-verde-oscuro">
                {delivery.version}
              </dd>
            </div>
            <div>
              <dt className="text-verde-oscuro/65">Último registro</dt>
              <dd className="font-semibold text-verde-oscuro">
                {formatDeliveryDate(delivery.updated_at)}
              </dd>
            </div>
            <div>
              <dt className="text-verde-oscuro/65">Nota actual</dt>
              <dd className="font-semibold text-verde-oscuro">
                {delivery.evaluation
                  ? `${formatScore(delivery.evaluation.score)} / 5.0`
                  : 'Sin calificar'}
              </dd>
            </div>
          </dl>
          <button
            className="mt-3 rounded-lg bg-white px-4 py-2 font-semibold text-verde-oscuro shadow-sm"
            onClick={() => void handleDownload()}
            type="button"
          >
            Descargar archivo actual
          </button>
        </div>

        {conflict && (
          <p className="mt-4 rounded-lg bg-crema p-4 font-semibold text-verde-oscuro" role="alert">
            La entrega cambió: el estudiante reemplazó el archivo. Revisa la
            versión actual y vuelve a calificar; la nota no se guardó.
          </p>
        )}

        {downloadError && (
          <p className="mt-4 rounded-lg bg-crema p-4 text-verde-oscuro" role="alert">
            {downloadError}
          </p>
        )}

        <form className="mt-6 grid gap-4" noValidate onSubmit={handleSubmit}>
          <div className="grid gap-1">
            <label className="font-medium" htmlFor="evaluation-score">
              Nota (0,0 a 5,0)
            </label>
            <input
              className="w-40 rounded-lg border border-verde-medio/30 bg-crema px-3 py-2"
              defaultValue={
                delivery.evaluation
                  ? formatScore(delivery.evaluation.score)
                  : ''
              }
              id="evaluation-score"
              key={delivery.version}
              max="5"
              min="0"
              name="score"
              required
              step="0.1"
              type="number"
            />
          </div>

          <div className="grid gap-1">
            <label className="font-medium" htmlFor="evaluation-feedback">
              Retroalimentación <span className="text-sm font-normal text-verde-oscuro/70">(opcional)</span>
            </label>
            <textarea
              className="min-h-32 rounded-lg border border-verde-medio/30 bg-crema px-3 py-2"
              defaultValue={delivery.evaluation?.feedback ?? ''}
              id="evaluation-feedback"
              key={`feedback-${delivery.version}`}
              maxLength={5000}
              name="feedback"
            />
          </div>

          {formError && !conflict && (
            <p className="rounded-lg bg-crema p-3 text-verde-oscuro" role="alert">
              {formError}
            </p>
          )}

          <button
            className="justify-self-start rounded-lg bg-dorado px-6 py-3 font-semibold text-verde-oscuro transition hover:brightness-110 disabled:opacity-60"
            disabled={mutation.isPending}
            type="submit"
          >
            {mutation.isPending ? 'Guardando...' : 'Guardar calificación'}
          </button>
        </form>
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
