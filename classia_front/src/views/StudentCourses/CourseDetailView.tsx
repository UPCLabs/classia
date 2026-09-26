import { useQuery } from '@tanstack/react-query'
import { Link, useParams } from 'react-router-dom'
import httpClient from '../../api/httpClient'
import { useAuth } from '../../auth/context'
import getErrorMessage from '../../auth/getErrorMessage'
import type { Curso } from '../../types/api'

export default function CourseDetailView() {
  const { courseId } = useParams()
  const { user } = useAuth()
  const courseQuery = useQuery({
    queryKey: ['course', courseId],
    enabled: Boolean(courseId),
    queryFn: async () => {
      const { data } = await httpClient.get<Curso>(`/courses/${courseId}`)
      return data
    },
  })

  if (courseQuery.isLoading) {
    return <p className="p-10 text-center" role="status">Cargando curso...</p>
  }

  if (courseQuery.error) {
    return (
      <main className="mx-auto max-w-3xl px-4 py-10">
        <p className="rounded-lg bg-red-50 p-4 text-red-800" role="alert">
          {getErrorMessage(courseQuery.error)}
        </p>
        <Link className="mt-4 inline-block underline" to="/courses">
          Volver a cursos
        </Link>
      </main>
    )
  }

  const course = courseQuery.data
  if (!course) return null

  const canEditCourse =
    user?.role === 'Teacher' && user.id === course.teacher.id

  return (
    <main className="mx-auto max-w-3xl px-4 py-10">
      <Link className="text-verde-medio underline" to="/courses">
        Volver a cursos
      </Link>
      <section className="mt-5 rounded-2xl bg-white p-6 shadow-sm sm:p-8">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
          <div>
            <p className="text-sm font-semibold uppercase tracking-wider text-verde-medio">
              {course.code}
            </p>
            <h1 className="mt-2 text-3xl font-bold">{course.name}</h1>
          </div>
          <span className="rounded-full bg-crema px-3 py-1 text-sm font-semibold">
            {course.status === 'active' ? 'Activo' : 'Inactivo'}
          </span>
        </div>

        <dl className="mt-8 grid gap-5 sm:grid-cols-2">
          <div>
            <dt className="text-sm text-verde-oscuro/65">Docente</dt>
            <dd className="mt-1 font-semibold">{course.teacher.name}</dd>
            <dd className="text-sm text-verde-oscuro/70">
              {course.teacher.email}
            </dd>
          </div>
          <div>
            <dt className="text-sm text-verde-oscuro/65">Cupo</dt>
            <dd className="mt-1 font-semibold">{course.capacity}</dd>
          </div>
          <div>
            <dt className="text-sm text-verde-oscuro/65">Estudiantes inscritos</dt>
            <dd className="mt-1 font-semibold">{course.enrolled_count}</dd>
          </div>
        </dl>

        {canEditCourse && (
          <Link
            className="mt-8 inline-flex rounded-lg bg-dorado px-5 py-3 font-semibold text-verde-oscuro transition hover:brightness-110"
            to={`/courses/${course.id}/edit`}
          >
            Editar curso
          </Link>
        )}
      </section>
    </main>
  )
}
