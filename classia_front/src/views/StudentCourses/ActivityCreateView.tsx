import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { createActivity } from '../../api/activities'
import httpClient from '../../api/httpClient'
import { useAuth } from '../../auth/context'
import getErrorMessage from '../../auth/getErrorMessage'
import type { ActivityCreate, Curso } from '../../types/api'
import ActivityForm from './ActivityForm'

export default function ActivityCreateView() {
  const { courseId } = useParams()
  const { user } = useAuth()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const [submitError, setSubmitError] = useState<string | null>(null)
  const courseQuery = useQuery({
    queryKey: ['course', courseId],
    enabled: Boolean(courseId),
    queryFn: async () => {
      const { data } = await httpClient.get<Curso>(`/courses/${courseId}`)
      return data
    },
  })
  const createMutation = useMutation({
    mutationFn: (values: ActivityCreate) => {
      if (!courseId) throw new Error('El curso no es válido.')
      return createActivity(courseId, values)
    },
  })

  async function handleSubmit(values: ActivityCreate) {
    setSubmitError(null)
    try {
      const activity = await createMutation.mutateAsync(values)
      await queryClient.invalidateQueries({
        queryKey: ['activities', courseId],
      })
      navigate(`/courses/${courseId}`, {
        replace: true,
        state: { message: `La actividad ${activity.title} se creó correctamente.` },
      })
    } catch (error: unknown) {
      setSubmitError(getErrorMessage(error))
    }
  }

  if (courseQuery.isLoading) {
    return <p className="p-10 text-center" role="status">Cargando curso...</p>
  }

  if (courseQuery.error) {
    return (
      <main className="mx-auto max-w-xl px-4 py-10">
        <p className="rounded-lg bg-crema p-4 text-verde-oscuro" role="alert">
          {getErrorMessage(courseQuery.error)}
        </p>
      </main>
    )
  }

  const course = courseQuery.data
  if (!course || !user) return null

  if (user.role !== 'Teacher' || user.id !== course.teacher.id) {
    return (
      <main className="mx-auto max-w-xl px-4 py-10">
        <p className="rounded-lg bg-crema p-4 text-verde-oscuro" role="alert">
          Solo el docente responsable puede crear actividades en este curso.
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
        <h1 className="mt-3 text-3xl font-bold">Crear actividad</h1>
        <p className="mt-2 text-verde-oscuro/70">{course.name}</p>
        <p className="mt-4 max-w-2xl text-verde-oscuro/80">
          Configura la información y la fecha límite. Podrás publicarla cuando
          esté lista.
        </p>
      </div>
      <ActivityForm
        isSubmitting={createMutation.isPending}
        onSubmit={handleSubmit}
        submitError={submitError}
        submitLabel="Crear actividad"
      />
    </main>
  )
}
