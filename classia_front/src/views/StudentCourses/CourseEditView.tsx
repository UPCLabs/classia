import { useEffect, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useNavigate, useParams } from 'react-router-dom'
import httpClient from '../../api/httpClient'
import { useAuth } from '../../auth/context'
import getErrorMessage from '../../auth/getErrorMessage'
import type { Curso, ListResponse, Usuario } from '../../types/api'
import CourseForm, { type CourseFormValues } from './CourseForm'

type CourseUpdate = Partial<CourseFormValues>

const courseFormValues = (course: Curso): CourseFormValues => ({
  name: course.name,
  code: course.code,
  teacher_id: course.teacher.id,
  status: course.status,
  capacity: course.capacity,
})

export default function CourseEditView() {
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
  const teachersQuery = useQuery({
    queryKey: ['course-teachers'],
    enabled: Boolean(user && user.role !== 'Teacher'),
    queryFn: async () => {
      const { data } = await httpClient.get<ListResponse<Usuario>>(
        '/users?role=Teacher&status=active',
      )
      return data.items
    },
  })
  const updateMutation = useMutation({
    mutationFn: async (values: CourseUpdate) => {
      const { data } = await httpClient.patch<Curso>(
        `/courses/${courseId}`,
        values,
      )
      return data
    },
    onSuccess: async (course) => {
      queryClient.setQueryData(['course', courseId], course)
      await queryClient.invalidateQueries({ queryKey: ['courses'] })
      navigate(`/courses/${courseId}`, { replace: true })
    },
  })

  useEffect(() => {
    if (!courseQuery.data || !user) return

    const isCourseTeacher =
      user.role === 'Teacher' && courseQuery.data.teacher.id === user.id
    const isAdmin = user.role === 'Admin' || user.role === 'SuperAdmin'
    if (!isCourseTeacher && !isAdmin) {
      navigate(`/courses/${courseId}`, { replace: true })
    }
  }, [courseId, courseQuery.data, navigate, user])

  async function handleSubmit(values: CourseFormValues) {
    const course = courseQuery.data
    if (!course) return

    const original = courseFormValues(course)
    const changes: CourseUpdate = {}
    if (values.name !== original.name) changes.name = values.name
    if (values.code !== original.code) changes.code = values.code
    if (values.teacher_id !== original.teacher_id) {
      changes.teacher_id = values.teacher_id
    }
    if (values.status !== original.status) changes.status = values.status
    if (values.capacity !== original.capacity) {
      changes.capacity = values.capacity
    }

    if (Object.keys(changes).length === 0) {
      setSubmitError('No cambiaste ningún dato del curso.')
      return
    }

    setSubmitError(null)
    try {
      await updateMutation.mutateAsync(changes)
    } catch (error: unknown) {
      setSubmitError(getErrorMessage(error))
    }
  }

  if (courseQuery.isLoading || (user?.role !== 'Teacher' && teachersQuery.isLoading)) {
    return <p className="p-10 text-center" role="status">Cargando curso...</p>
  }

  if (courseQuery.error) {
    return (
      <main className="mx-auto max-w-xl px-4 py-10">
        <p className="rounded-lg bg-red-50 p-4 text-red-800" role="alert">
          {getErrorMessage(courseQuery.error)}
        </p>
        <Link className="mt-4 inline-block underline" to="/courses">
          Volver a cursos
        </Link>
      </main>
    )
  }

  if (teachersQuery.error) {
    return (
      <main className="mx-auto max-w-xl px-4 py-10">
        <p className="rounded-lg bg-red-50 p-4 text-red-800" role="alert">
          {getErrorMessage(teachersQuery.error)}
        </p>
      </main>
    )
  }

  const course = courseQuery.data
  if (!course || !user) return null

  const isTeacher = user.role === 'Teacher'
  const teachers = teachersQuery.data ?? []
  const assignedTeacher = isTeacher
    ? {
        id: user.id,
        name: user.name,
        email: user.email,
        role: user.role,
        status: 'active' as const,
        created_at: '',
        updated_at: '',
      }
    : undefined

  return (
    <main className="mx-auto max-w-2xl px-4 py-10">
      <div className="mb-6">
        <Link
          className="text-verde-medio underline"
          to={`/courses/${course.id}`}
        >
          Volver al detalle
        </Link>
        <h1 className="mt-3 text-3xl font-bold">Editar curso</h1>
      </div>
      <CourseForm
        assignedTeacher={assignedTeacher}
        initialValues={courseFormValues(course)}
        isSubmitting={updateMutation.isPending}
        onSubmit={handleSubmit}
        submitError={submitError}
        submitLabel="Guardar cambios"
        teacherLocked={isTeacher}
        teachers={teachers}
      />
    </main>
  )
}
