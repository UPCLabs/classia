import { useState } from 'react'
import { useMutation, useQuery } from '@tanstack/react-query'
import { Link } from 'react-router-dom'
import httpClient from '../../api/httpClient'
import { useAuth } from '../../auth/context'
import getErrorMessage from '../../auth/getErrorMessage'
import type { ListResponse, Usuario } from '../../types/api'
import CourseForm, { type CourseFormValues } from './CourseForm'

type CreatedCourse = {
  id: string
  name: string
  code: string
}

const emptyCourse: CourseFormValues = {
  name: '',
  code: '',
  teacher_id: '',
  password: '',
  status: 'active',
  capacity: 1,
}

export default function CourseCreateView() {
  const { user } = useAuth()
  const [submitError, setSubmitError] = useState<string | null>(null)
  const [createdCourse, setCreatedCourse] = useState<CreatedCourse | null>(null)
  const isTeacher = user?.role === 'Teacher'
  const teachersQuery = useQuery({
    queryKey: ['course-teachers'],
    enabled: Boolean(user && !isTeacher),
    queryFn: async () => {
      const { data } = await httpClient.get<ListResponse<Usuario>>(
        '/users?role=Teacher&status=active',
      )
      return data.items
    },
  })
  const createMutation = useMutation({
    mutationFn: async (values: CourseFormValues) => {
      const { data } = await httpClient.post<CreatedCourse>('/courses/create', {
        name: values.name,
        code: values.code,
        teacher_id: isTeacher ? user.id : values.teacher_id,
        password: values.password,
        status: values.status,
        quantity: values.capacity,
      })
      return data
    },
  })

  async function handleSubmit(values: CourseFormValues) {
    setSubmitError(null)
    try {
      const course = await createMutation.mutateAsync(values)
      setCreatedCourse(course)
    } catch (error: unknown) {
      setSubmitError(getErrorMessage(error))
    }
  }

  if (!user) return null

  if (teachersQuery.isLoading) {
    return <p className="p-10 text-center" role="status">Cargando docentes...</p>
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
        <Link className="text-verde-medio underline" to="/courses">
          Volver a cursos
        </Link>
        <h1 className="mt-3 text-3xl font-bold">Crear curso</h1>
      </div>
      {createdCourse ? (
        <section
          className="rounded-xl bg-white p-6 shadow-sm"
          role="status"
        >
          <h2 className="text-xl font-bold">Curso creado correctamente</h2>
          <p className="mt-2">{createdCourse.name} ({createdCourse.code})</p>
          <p className="mt-1 text-sm text-verde-oscuro/70">
            Identificador: {createdCourse.id}
          </p>
        </section>
      ) : (
        <CourseForm
          assignedTeacher={assignedTeacher}
          initialValues={{
            ...emptyCourse,
            teacher_id: isTeacher ? user.id : '',
          }}
          isSubmitting={createMutation.isPending}
          onSubmit={handleSubmit}
          requireCoursePassword
          submitError={submitError}
          submitLabel="Crear curso"
          teacherLocked={isTeacher}
          teachers={teachers}
        />
      )}
    </main>
  )
}
