import { useQuery } from '@tanstack/react-query'
import { Link, useLocation, useParams } from 'react-router-dom'
import { getActivity, isActivityDue } from '../../api/activities'
import { getMyDelivery } from '../../api/deliveries'
import getErrorMessage from '../../auth/getErrorMessage'
import {
  deliveryStatusLabel,
  formatDeliveryDate,
  formatScore,
} from './deliveryPresentation'

export default function DeliveryStatusView() {
  const { activityId } = useParams()
  const location = useLocation()
  const locationState = location.state as { message?: string } | null

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
  const isOpen = !isActivityDue(activity.due_at)

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
          Estado de la entrega
        </p>
        <h1 className="mt-2 text-3xl font-bold text-verde-oscuro">
          {activity.title}
        </h1>

        {locationState?.message && (
          <p className="mt-4 rounded-lg bg-crema p-4 text-verde-oscuro" role="status">
            {locationState.message}
          </p>
        )}

        <div className="mt-6 flex flex-wrap items-center gap-3">
          <span className="rounded-full bg-crema px-4 py-1 font-semibold text-verde-oscuro">
            {deliveryStatusLabel(delivery?.status ?? null)}
          </span>
          {delivery && (
            <span className="text-sm text-verde-oscuro/70">
              Versión {delivery.version}
            </span>
          )}
        </div>

        {delivery ? (
          <>
            <dl className="mt-6 grid gap-4 border-t border-verde-medio/25 pt-5 sm:grid-cols-2">
              <div>
                <dt className="text-sm text-verde-oscuro/65">Entregada</dt>
                <dd className="mt-1 font-semibold text-verde-oscuro">
                  {formatDeliveryDate(delivery.submitted_at)}
                </dd>
              </div>
              <div>
                <dt className="text-sm text-verde-oscuro/65">
                  Última actualización
                </dt>
                <dd className="mt-1 font-semibold text-verde-oscuro">
                  {formatDeliveryDate(delivery.updated_at)}
                </dd>
              </div>
            </dl>

            <section
              aria-labelledby="delivery-evaluation-heading"
              className="mt-6 rounded-xl border border-verde-medio/25 p-5"
            >
              <h2
                className="text-xl font-semibold text-verde-oscuro"
                id="delivery-evaluation-heading"
              >
                Calificación
              </h2>
              {delivery.evaluation ? (
                <>
                  <p className="mt-3 text-2xl font-bold text-verde-oscuro">
                    {formatScore(delivery.evaluation.score)} / 5.0
                  </p>
                  <p className="mt-3 text-sm text-verde-oscuro/65">
                    Retroalimentación
                  </p>
                  <p className="mt-1 whitespace-pre-wrap text-verde-oscuro/85">
                    {delivery.evaluation.feedback ?? 'Sin retroalimentación'}
                  </p>
                </>
              ) : (
                <p className="mt-3 text-verde-oscuro/75">
                  La entrega aún no tiene calificación.
                </p>
              )}
            </section>

            {isOpen && (
              <Link
                className="mt-6 inline-block rounded-lg bg-dorado px-5 py-2 font-semibold text-verde-oscuro"
                to={`/activities/${activityId}/delivery`}
              >
                Reemplazar entrega
              </Link>
            )}
          </>
        ) : (
          <div className="mt-6 border-t border-verde-medio/25 pt-5">
            <p className="text-verde-oscuro/80">
              Todavía no has subido una entrega para esta actividad.
            </p>
            {isOpen ? (
              <Link
                className="mt-4 inline-block rounded-lg bg-dorado px-5 py-2 font-semibold text-verde-oscuro"
                to={`/activities/${activityId}/delivery`}
              >
                Subir entrega
              </Link>
            ) : (
              <p className="mt-4 font-semibold text-verde-oscuro" role="alert">
                El plazo de entrega ya venció.
              </p>
            )}
          </div>
        )}
      </section>
    </main>
  )
}
