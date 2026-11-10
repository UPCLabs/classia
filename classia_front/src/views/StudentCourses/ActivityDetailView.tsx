import { useQuery } from '@tanstack/react-query'
import { Link, useParams } from 'react-router-dom'
import { getActivity, isActivityDue } from '../../api/activities'
import { getMyDelivery } from '../../api/deliveries'
import { useAuth } from '../../auth/context'
import getErrorMessage from '../../auth/getErrorMessage'
import { deliveryStatusLabel } from './deliveryPresentation'

const bogotaDateFormatter = new Intl.DateTimeFormat('es-CO', {
  dateStyle: 'long',
  timeStyle: 'short',
  timeZone: 'America/Bogota',
})

export default function ActivityDetailView() {
  const { activityId } = useParams()
  const { user } = useAuth()
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
      <article className="mt-5 rounded-2xl border-l-4 border-dorado bg-crema/40 p-6 shadow-sm sm:p-8">
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
    </main>
  )
}
