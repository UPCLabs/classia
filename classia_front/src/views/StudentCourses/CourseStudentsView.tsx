import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useParams } from 'react-router-dom'
import httpClient from '../../api/httpClient'
import { useAuth } from '../../auth/context'
import getErrorMessage from '../../auth/getErrorMessage'
import type { Curso, Inscripcion, ListResponse, Usuario } from '../../types/api'

export default function CourseStudentsView() {
  const { courseId } = useParams()
  const { user } = useAuth()
  const queryClient = useQueryClient()
  const [isAddingStudent, setIsAddingStudent] = useState(false)
  const [search, setSearch] = useState('')
  const [successMessage, setSuccessMessage] = useState<string | null>(null)
  const courseQuery = useQuery({
    queryKey: ['course', courseId],
    enabled: Boolean(courseId),
    queryFn: async () => {
      const { data } = await httpClient.get<Curso>(`/courses/${courseId}`)
      return data
    },
  })
  const studentsQuery = useQuery({
    queryKey: ['course', courseId, 'students'],
    enabled: Boolean(courseId),
    queryFn: async () => {
      const { data } = await httpClient.get<ListResponse<Usuario>>(
        `/courses/${courseId}/students`,
      )
      return data.items
    },
  })
  const isCourseTeacher =
    user?.role === 'Teacher' && user.id === courseQuery.data?.teacher.id
  const availableStudentsQuery = useQuery({
    queryKey: ['course', courseId, 'available-students', search.trim()],
    enabled: Boolean(courseId && isCourseTeacher && isAddingStudent),
    queryFn: async () => {
      const query = search.trim()
      const suffix = query ? `?q=${encodeURIComponent(query)}` : ''
      const { data } = await httpClient.get<ListResponse<Usuario>>(
        `/courses/${courseId}/available-students${suffix}`,
      )
      return data.items
    },
  })
  const enrollMutation = useMutation({
    mutationFn: async (studentId: string) => {
      const { data } = await httpClient.post<Inscripcion>(
        `/courses/${courseId}/students`,
        { student_id: studentId },
      )
      return data
    },
    onSuccess: async () => {
      await Promise.all([
        queryClient.invalidateQueries({
          queryKey: ['course', courseId, 'students'],
        }),
        queryClient.invalidateQueries({
          queryKey: ['course', courseId, 'available-students'],
        }),
        queryClient.invalidateQueries({ queryKey: ['course', courseId] }),
        queryClient.invalidateQueries({ queryKey: ['courses'] }),
      ])
      setSuccessMessage('Estudiante inscrito correctamente.')
    },
  })

  if (courseQuery.isLoading || studentsQuery.isLoading) {
    return <p className="p-10 text-center" role="status">Cargando participantes...</p>
  }

  if (courseQuery.error) {
    return (
      <main className="mx-auto max-w-4xl px-4 py-10">
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

  const students = studentsQuery.data ?? []
  const isAdmin = user?.role === 'Admin' || user?.role === 'SuperAdmin'
  const canEnrollStudents = isCourseTeacher

  return (
    <main className="mx-auto max-w-4xl px-4 py-10 sm:px-6">
      <Link
        className="text-verde-medio underline"
        to={`/courses/${course.id}`}
      >
        Volver al curso
      </Link>
      <header className="mt-4">
        <p className="text-sm font-semibold uppercase tracking-wider text-verde-medio">
          {course.code}
        </p>
        <h1 className="mt-2 text-3xl font-bold">{course.name} · Participantes</h1>
        <p className="mt-2 font-medium">
          Cupo: {course.enrolled_count} / {course.capacity}
        </p>
        {course.enrolled_count >= course.capacity && (
          <p className="mt-2 text-sm text-red-800" role="status">
            Cupo completo. El backend validará las nuevas inscripciones.
          </p>
        )}
        {course.status !== 'active' && (
          <p className="mt-2 text-sm text-verde-oscuro/70" role="status">
            El curso está inactivo; el backend validará las inscripciones.
          </p>
        )}
      </header>

      {studentsQuery.error ? (
        <p className="mt-6 rounded-lg bg-red-50 p-4 text-red-800" role="alert">
          {getErrorMessage(studentsQuery.error)}
        </p>
      ) : (
        <section className="mt-7 rounded-xl bg-white p-6 shadow-sm">
          <h2 className="text-xl font-bold">Estudiantes inscritos</h2>
          {students.length === 0 ? (
            <p className="mt-4 text-verde-oscuro/70">
              Aún no hay estudiantes inscritos.
            </p>
          ) : (
            <ul className="mt-4 divide-y divide-verde-oscuro/10">
              {students.map((student) => (
                <li className="py-3" key={student.id}>
                  <p className="font-semibold">{student.name}</p>
                  <p className="text-sm text-verde-oscuro/70">
                    {student.email}
                  </p>
                </li>
              ))}
            </ul>
          )}
        </section>
      )}

      {isAdmin && (
        <p className="mt-5 text-sm text-verde-oscuro/70">
          Los administradores pueden consultar a los participantes, pero no inscribir estudiantes.
        </p>
      )}

      {canEnrollStudents && (
        <section className="mt-7">
          <button
            aria-expanded={isAddingStudent}
            className="rounded-lg bg-dorado px-5 py-3 font-semibold text-verde-oscuro"
            onClick={() => setIsAddingStudent((isOpen) => !isOpen)}
            type="button"
          >
            Agregar estudiante
          </button>

          {isAddingStudent && (
            <div className="mt-4 rounded-xl bg-white p-6 shadow-sm">
              <label className="grid gap-1 font-medium" htmlFor="student-search">
                Buscar estudiantes disponibles
                <input
                  className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2 font-normal"
                  id="student-search"
                  onChange={(event) => setSearch(event.target.value)}
                  placeholder="Nombre o correo"
                  type="search"
                  value={search}
                />
              </label>

              {enrollMutation.error && (
                <p className="mt-4 rounded-lg bg-red-50 p-3 text-red-800" role="alert">
                  {getErrorMessage(enrollMutation.error)}
                </p>
              )}
              {successMessage && (
                <p className="mt-4 rounded-lg bg-green-50 p-3 text-green-800" role="status">
                  {successMessage}
                </p>
              )}

              {availableStudentsQuery.isLoading ? (
                <p className="mt-4" role="status">Buscando estudiantes...</p>
              ) : availableStudentsQuery.error ? (
                <p className="mt-4 rounded-lg bg-red-50 p-3 text-red-800" role="alert">
                  {getErrorMessage(availableStudentsQuery.error)}
                </p>
              ) : (availableStudentsQuery.data ?? []).length === 0 ? (
                <p className="mt-4 text-verde-oscuro/70">
                  No hay estudiantes disponibles.
                </p>
              ) : (
                <ul className="mt-4 divide-y divide-verde-oscuro/10">
                  {availableStudentsQuery.data?.map((student) => (
                    <li
                      className="flex flex-col gap-3 py-3 sm:flex-row sm:items-center sm:justify-between"
                      key={student.id}
                    >
                      <div>
                        <p className="font-semibold">{student.name}</p>
                        <p className="text-sm text-verde-oscuro/70">
                          {student.email}
                        </p>
                      </div>
                      <button
                        className="rounded-lg border border-verde-medio px-4 py-2 font-semibold text-verde-medio disabled:opacity-60"
                        disabled={enrollMutation.isPending}
                        onClick={() => {
                          setSuccessMessage(null)
                          enrollMutation.mutate(student.id)
                        }}
                        type="button"
                      >
                        Inscribir
                      </button>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          )}
        </section>
      )}
    </main>
  )
}
