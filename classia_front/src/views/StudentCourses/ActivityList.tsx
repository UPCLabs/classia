import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import { listCourseActivities } from '../../api/activities'
import { useAuth } from '../../auth/context'
import getErrorMessage from '../../auth/getErrorMessage'
import type { Activity } from '../../types/api'
import PublishActivityButton from './PublishActivityButton'

type ActivityListProps = {
  courseId: string
  canPublish: boolean
}

const bogotaDateFormatter = new Intl.DateTimeFormat('es-CO', {
  dateStyle: 'long',
  timeStyle: 'short',
  timeZone: 'America/Bogota',
})

function formatDueDate(value: string): string {
  return bogotaDateFormatter.format(new Date(value))
}

export default function ActivityList({ courseId, canPublish }: ActivityListProps) {
  const { user } = useAuth()
  const activitiesQuery = useQuery({
    queryKey: ['activities', courseId],
    queryFn: () => listCourseActivities(courseId),
  })

  if (activitiesQuery.isLoading) {
    return <p className="mt-8 text-center" role="status">Cargando actividades...</p>
  }

  if (activitiesQuery.error) {
    return (
      <p className="mt-8 rounded-lg bg-crema p-4 text-verde-oscuro" role="alert">
        {getErrorMessage(activitiesQuery.error)}
      </p>
    )
  }

  const activities = activitiesQuery.data ?? []
  const visibleActivities =
    user?.role === 'Student'
      ? activities.filter((activity) => activity.status === 'published')
      : activities

  return (
    <section aria-labelledby="course-activities-heading" className="mt-8">
      <div>
        <h2
          className="text-2xl font-bold text-verde-oscuro"
          id="course-activities-heading"
        >
          Actividades
        </h2>
        <p className="mt-1 text-verde-oscuro/70">
          Consulta las actividades y sus fechas límite.
        </p>
      </div>

      {visibleActivities.length === 0 ? (
        <p className="mt-5 rounded-xl bg-verde-medio p-6 text-center text-crema">
          No hay actividades para mostrar.
        </p>
      ) : (
        <ul className="mt-5 grid gap-4">
          {visibleActivities.map((activity) => (
            <ActivityListItem
              activity={activity}
              canPublish={canPublish}
              key={activity.id}
              showStatus={user?.role !== 'Student'}
            />
          ))}
        </ul>
      )}
    </section>
  )
}

type ActivityListItemProps = {
  activity: Activity
  canPublish: boolean
  showStatus: boolean
}

function ActivityListItem({
  activity,
  canPublish,
  showStatus,
}: ActivityListItemProps) {
  return (
    <li className="rounded-xl border-l-4 border-dorado bg-white p-5">
      <Link
        className="block rounded-lg transition hover:text-verde-medio focus-visible:outline focus-visible:outline-2 focus-visible:outline-verde-medio"
        to={`/activities/${activity.id}`}
      >
        <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
          <div>
            <h3 className="text-lg font-semibold text-verde-oscuro">
              {activity.title}
            </h3>
            <p className="mt-2 line-clamp-2 text-sm text-verde-oscuro/75">
              {activity.description}
            </p>
          </div>
          {showStatus && (
            <span className="w-fit rounded-full bg-crema px-3 py-1 text-sm font-semibold text-verde-oscuro">
              {activity.status === 'draft' ? 'Borrador' : 'Publicada'}
            </span>
          )}
        </div>
        <p className="mt-4 text-sm text-verde-oscuro/75">
          Fecha límite: {formatDueDate(activity.due_at)} (Bogotá)
        </p>
      </Link>
      {canPublish && (
        <div className="mt-4 flex flex-wrap items-center gap-3">
          <Link
            className="rounded-lg border border-verde-medio px-4 py-2 font-semibold text-verde-medio transition hover:bg-verde-medio hover:text-crema"
            to={`/activities/${activity.id}/edit`}
          >
            Editar actividad
          </Link>
          <PublishActivityButton activity={activity} />
        </div>
      )}
    </li>
  )
}
