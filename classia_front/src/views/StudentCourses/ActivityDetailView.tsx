import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Link, useLocation, useParams } from 'react-router-dom'
import { getActivity, isActivityDue } from '../../api/activities'
import {
  downloadDeliveryFile,
  getMyDelivery,
  listReceivedDeliveries,
} from '../../api/deliveries'
import { useAuth } from '../../auth/context'
import getErrorMessage from '../../auth/getErrorMessage'
import { saveBlob } from '../../shared/downloadBlob'
import type { Entrega } from '../../types/api'
import {
  deliveryStatusLabel,
  formatDeliveryDate,
  formatScore,
} from './deliveryPresentation'

const bogotaDateFormatter = new Intl.DateTimeFormat('es-CO', {
  dateStyle: 'long',
  timeStyle: 'short',
  timeZone: 'America/Bogota',
})

export default function ActivityDetailView() {
  const { activityId } = useParams()
  const { user } = useAuth()
  const location = useLocation()
  const locationState = location.state as { message?: string } | null
  const [downloadError, setDownloadError] = useState<string | null>(null)
  const activityQuery = useQuery({
    queryKey: ['activity', activityId],
    enabled: Boolean(activityId),
    queryFn: () => {
      if (!activityId) throw new Error('La actividad no es válida.')
      return getActivity(activityId)
    },
  })

  const isStudent = user?.role === 'Student'
  const deliveryQuery = useQuery({
    queryKey: ['delivery', activityId],
    enabled:
      isStudent &&
      Boolean(activityId) &&
      activityQuery.data?.status === 'published',
    queryFn: () => {
      if (!activityId) throw new Error('La actividad no es válida.')
      return getMyDelivery(activityId)
    },
  })

  const isTeacher = user?.role === 'Teacher'
  const deliveriesQuery = useQuery({
    queryKey: ['deliveries', activityId],
    enabled:
      isTeacher &&
      Boolean(activityId) &&
      activityQuery.data?.status === 'published',
    queryFn: () => {
      if (!activityId) throw new Error('La actividad no es válida.')
      return listReceivedDeliveries(activityId)
    },
  })

  async function handleDownload(delivery: Entrega) {
    setDownloadError(null)
    try {
      const blob = await downloadDeliveryFile(delivery.id)
      saveBlob(blob, `entrega-${delivery.id}.docx`)
    } catch (error) {
      setDownloadError(getErrorMessage(error))
    }
  }

  if (activityQuery.isLoading) {
    return <p className="p-10 text-center" role="status">Cargando actividad...</p>
  }

  if (activityQuery.error) {
    return (
      <main className="mx-auto max-w-3xl px-4 py-10">
        <p className="rounded-lg bg-crema p-4 text-verde-oscuro" role="alert">
          {getErrorMessage(activityQuery.error)}
        </p>
        <Link className="mt-4 inline-block text-verde-medio underline" to="/courses">
          Volver a cursos
        </Link>
      </main>
    )
  }

  const activity = activityQuery.data
  if (!activity) return null

  if (user?.role === 'Student' && activity.status !== 'published') {
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

  return (
    <main className="mx-auto max-w-3xl px-4 py-10">
      <Link
        className="text-verde-medio underline"
        to={`/courses/${activity.course_id}`}
      >
        Volver al curso
      </Link>
      {locationState?.message && (
        <p className="mt-4 rounded-lg bg-crema p-4 text-verde-oscuro" role="status">
          {locationState.message}
        </p>
      )}
      <article className="mt-5 rounded-2xl border-l-4 border-dorado bg-white p-6 shadow-sm sm:p-8">
        <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
          <h1 className="text-3xl font-bold text-verde-oscuro">
            {activity.title}
          </h1>
          {user?.role !== 'Student' && (
            <span className="w-fit rounded-full bg-crema px-3 py-1 text-sm font-semibold text-verde-oscuro">
              {activity.status === 'draft' ? 'Borrador' : 'Publicada'}
            </span>
          )}
        </div>
        <p className="mt-6 whitespace-pre-wrap text-verde-oscuro/85">
          {activity.description}
        </p>
        <dl className="mt-8 border-t border-verde-medio/25 pt-5">
          <dt className="text-sm text-verde-oscuro/65">Fecha límite</dt>
          <dd className="mt-1 font-semibold text-verde-oscuro">
            {bogotaDateFormatter.format(new Date(activity.due_at))} (Bogotá)
          </dd>
        </dl>
      </article>

      {isStudent && activity.status === 'published' && (
        <section
          aria-labelledby="student-delivery-heading"
          className="mt-6 rounded-2xl bg-white p-6 shadow-sm sm:p-8"
        >
          <h2
            className="text-2xl font-bold text-verde-oscuro"
            id="student-delivery-heading"
          >
            Tu entrega
          </h2>

          {deliveryQuery.isLoading ? (
            <p className="mt-3" role="status">Cargando entrega...</p>
          ) : deliveryQuery.error ? (
            <p className="mt-3 rounded-lg bg-crema p-4 text-verde-oscuro" role="alert">
              {getErrorMessage(deliveryQuery.error)}
            </p>
          ) : (
            <>
              <p className="mt-3 text-verde-oscuro/80">
                Estado:{' '}
                <span className="font-semibold text-verde-oscuro">
                  {deliveryStatusLabel(deliveryQuery.data?.status ?? null)}
                </span>
              </p>
              <div className="mt-4 flex flex-wrap gap-3">
                <Link
                  className="rounded-lg bg-crema px-5 py-2 font-semibold text-verde-oscuro"
                  to={`/activities/${activityId}/delivery/status`}
                >
                  Ver estado de mi entrega
                </Link>
                {!isActivityDue(activity.due_at) && (
                  <Link
                    className="rounded-lg bg-dorado px-5 py-2 font-semibold text-verde-oscuro"
                    to={`/activities/${activityId}/delivery`}
                  >
                    {deliveryQuery.data ? 'Reemplazar entrega' : 'Subir entrega'}
                  </Link>
                )}
              </div>
            </>
          )}
        </section>
      )}

      {isTeacher && activity.status === 'published' && (
        <section
          aria-labelledby="teacher-deliveries-heading"
          className="mt-6 rounded-2xl bg-white p-6 shadow-sm sm:p-8"
        >
          <div className="flex flex-wrap items-baseline justify-between gap-2">
            <h2
              className="text-2xl font-bold text-verde-oscuro"
              id="teacher-deliveries-heading"
            >
              Entregas recibidas
            </h2>
            {deliveriesQuery.data && (
              <p className="text-sm font-semibold text-verde-oscuro/70">
                {deliveriesQuery.data.total}{' '}
                {deliveriesQuery.data.total === 1 ? 'entrega' : 'entregas'}
              </p>
            )}
          </div>

          {deliveriesQuery.isLoading ? (
            <p className="mt-3" role="status">Cargando entregas...</p>
          ) : deliveriesQuery.error ? (
            <p className="mt-3 rounded-lg bg-crema p-4 text-verde-oscuro" role="alert">
              {getErrorMessage(deliveriesQuery.error)}
            </p>
          ) : (deliveriesQuery.data?.items ?? []).length === 0 ? (
            <p className="mt-3 rounded-xl bg-crema p-5 text-center text-verde-oscuro/80">
              Aún no hay entregas para esta actividad
            </p>
          ) : (
            <ul className="mt-4 grid gap-3">
              {deliveriesQuery.data?.items.map((delivery) => (
                <li
                  className="rounded-xl border border-verde-medio/25 p-4"
                  key={delivery.id}
                >
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div>
                      <p className="font-semibold text-verde-oscuro">
                        {delivery.student.name}
                      </p>
                      <p className="text-sm text-verde-oscuro/70">
                        {delivery.student.email}
                      </p>
                    </div>
                    <span className="rounded-full bg-crema px-3 py-1 text-sm font-semibold text-verde-oscuro">
                      {deliveryStatusLabel(delivery.status)}
                    </span>
                  </div>

                  <dl className="mt-3 grid gap-2 text-sm sm:grid-cols-2">
                    <div>
                      <dt className="text-verde-oscuro/65">Fecha de registro</dt>
                      <dd className="font-semibold text-verde-oscuro">
                        {formatDeliveryDate(delivery.updated_at)}
                      </dd>
                    </div>
                    <div>
                      <dt className="text-verde-oscuro/65">Nota</dt>
                      <dd className="font-semibold text-verde-oscuro">
                        {delivery.evaluation
                          ? `${formatScore(delivery.evaluation.score)} / 5.0`
                          : 'Sin calificar'}
                      </dd>
                    </div>
                  </dl>

                  <div className="mt-4 flex flex-wrap gap-3">
                    <button
                      className="rounded-lg bg-crema px-4 py-2 font-semibold text-verde-oscuro"
                      onClick={() => void handleDownload(delivery)}
                      type="button"
                    >
                      Descargar
                    </button>
                    <Link
                      className="rounded-lg bg-dorado px-4 py-2 font-semibold text-verde-oscuro"
                      to={`/activities/${activityId}/deliveries/${delivery.id}/evaluation`}
                    >
                      {delivery.evaluation ? 'Corregir calificación' : 'Calificar'}
                    </Link>
                  </div>
                </li>
              ))}
            </ul>
          )}

          {downloadError && (
            <p className="mt-3 rounded-lg bg-crema p-4 text-verde-oscuro" role="alert">
              {downloadError}
            </p>
          )}
        </section>
      )}
    </main>
  )
}
