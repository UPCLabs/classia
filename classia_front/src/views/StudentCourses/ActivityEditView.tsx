import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { getActivity, updateActivity } from '../../api/activities'
import httpClient from '../../api/httpClient'
import { useAuth } from '../../auth/context'
import getErrorMessage from '../../auth/getErrorMessage'
import type { ActivityCreate, ActivityUpdate, Curso } from '../../types/api'
import ActivityForm from './ActivityForm'

export default function ActivityEditView() {
  const { activityId } = useParams()
  const { user } = useAuth()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [submitError, setSubmitError] = useState<string | null>(null)
  const activityQuery = useQuery({
    queryKey: ['activity', activityId],
    enabled: Boolean(activityId),
    queryFn: () => {
      if (!activityId) throw new Error('La actividad no es válida.')
      return getActivity(activityId)
    },
  })
  const courseQuery = useQuery({
    queryKey: ['course', activityQuery.data?.course_id],
    enabled: Boolean(activityQuery.data?.course_id),
    queryFn: async () => {
      const { data } = await httpClient.get<Curso>(
        `/courses/${activityQuery.data?.course_id}`,
      )
      return data
    },
  })
  const updateMutation = useMutation({
    mutationFn: (changes: ActivityUpdate) => {
      if (!activityId) throw new Error('La actividad no es válida.')
      return updateActivity(activityId, changes)
    },
  })

  async function handleSubmit(values: ActivityCreate) {
    const activity = activityQuery.data
    if (!activity) return

    const changes: ActivityUpdate = {}
    if (values.title !== activity.title) changes.title = values.title
    if (values.description !== activity.description) {
      changes.description = values.description
    }
    if (
      Math.floor(new Date(values.due_at).getTime() / 60_000) !==
      Math.floor(new Date(activity.due_at).getTime() / 60_000)
    ) {
      changes.due_at = values.due_at
    }

    if (Object.keys(changes).length === 0) {
      setSubmitError('No cambiaste ningún dato de la actividad.')
      return
    }

    setSubmitError(null)
    try {
      const updatedActivity = await updateMutation.mutateAsync(changes)
      queryClient.setQueryData(['activity', activityId], updatedActivity)
      await queryClient.invalidateQueries({
        queryKey: ['activities', activity.course_id],
      })
      navigate(`/courses/${activity.course_id}`, {
        replace: true,
        state: { message: 'La actividad se actualizó correctamente.' },
      })
    } catch (error: unknown) {
      setSubmitError(getErrorMessage(error))
    }
  }

  if (activityQuery.isLoading || courseQuery.isLoading) {
    return <p className="p-10 text-center" role="status">Cargando actividad...</p>
  }

  if (activityQuery.error || courseQuery.error) {
    const error = activityQuery.error ?? courseQuery.error
    return (
      <main className="mx-auto max-w-xl px-4 py-10">
        <p className="rounded-lg bg-crema p-4 text-verde-oscuro" role="alert">
          {getErrorMessage(error)}
        </p>
      </main>
    )
  }

  const activity = activityQuery.data
  const course = courseQuery.data
  if (!activity || !course || !user) return null

  if (user.role !== 'Teacher' || user.id !== course.teacher.id) {
    return (
      <main className="mx-auto max-w-xl px-4 py-10">
        <p className="rounded-lg bg-crema p-4 text-verde-oscuro" role="alert">
          Solo el docente responsable puede editar actividades de este curso.
        </p>
        <Link className="mt-4 inline-block text-verde-medio underline" to={`/courses/${course.id}`}>
          Volver al curso
        </Link>
      </main>
    )
  }

  return (
    <main className="mx-auto max-w-3xl px-4 py-10">
      <div className="mb-6">
        <Link
          className="text-verde-medio underline"
          to={`/courses/${course.id}`}
        >
          Volver al curso
        </Link>
        <h1 className="mt-3 text-3xl font-bold">Editar actividad</h1>
        <p className="mt-2 text-verde-oscuro/70">{course.name}</p>
        <p className="mt-4 max-w-2xl text-verde-oscuro/80">
          Actualiza la información o la fecha límite de esta actividad.
        </p>
      </div>
      <ActivityForm
        initialActivity={activity}
        isSubmitting={updateMutation.isPending}
        onSubmit={handleSubmit}
        submitError={submitError}
        submitLabel="Guardar cambios"
      />
    </main>
  )
}
