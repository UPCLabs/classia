import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import httpClient from '../../api/httpClient'
import { useAuth } from '../../auth/context'
import getErrorMessage from '../../auth/getErrorMessage'
import type { Curso, ListResponse } from '../../types/api'

export default function CoursesView() {
  const { user } = useAuth()
  const coursesQuery = useQuery({
    queryKey: ['courses'],
    queryFn: async () => {
      const { data } = await httpClient.get<ListResponse<Curso>>('/courses')
      return data.items
    },
  })

  if (coursesQuery.isLoading) {
    return <p className="p-10 text-center" role="status">Cargando cursos...</p>
  }

  if (coursesQuery.error) {
    return (
      <main className="mx-auto max-w-5xl px-4 py-10">
        <p className="rounded-lg bg-red-50 p-4 text-red-800" role="alert">
          {getErrorMessage(coursesQuery.error)}
        </p>
      </main>
    )
  }

  const courses = coursesQuery.data ?? []
  const canCreateCourse =
    user?.role === 'Teacher' ||
    user?.role === 'Admin' ||
    user?.role === 'SuperAdmin'

  return (
    <main className="mx-auto max-w-5xl px-4 py-10 sm:px-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <h1 className="text-3xl font-bold">Cursos</h1>
          <p className="mt-2 text-verde-oscuro/70">
            Consulta los cursos disponibles para tu cuenta.
          </p>
        </div>
        {canCreateCourse && (
          <Link
            className="rounded-lg bg-dorado px-5 py-3 text-center font-semibold text-verde-oscuro"
            to="/courses/new"
          >
            Crear curso
          </Link>
        )}
      </div>

      {courses.length === 0 ? (
        <p className="mt-8 rounded-xl bg-verde-medio p-6 text-center text-crema">
          No hay cursos para mostrar.
        </p>
      ) : (
        <div className="mt-8 grid gap-4 sm:grid-cols-2">
          {courses.map((course) => (
            <Link
              className="rounded-xl border-l-4 border-dorado bg-white p-5 shadow-sm transition hover:shadow-md"
              key={course.id}
              to={`/courses/${course.id}`}
            >
              <p className="text-sm font-semibold text-verde-medio">
                {course.code}
              </p>
              <h2 className="mt-1 text-xl font-bold">{course.name}</h2>
              <p className="mt-2 text-sm text-verde-oscuro/70">
                Docente: {course.teacher.name}
              </p>
            </Link>
          ))}
        </div>
      )}
    </main>
  )
}
