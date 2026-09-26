import { useState } from 'react'
import { useMutation, useQuery } from '@tanstack/react-query'
import { Link, useNavigate } from 'react-router-dom'
import httpClient from '../../api/httpClient'
import { useAuth } from '../../auth/context'
import getErrorMessage from '../../auth/getErrorMessage'
import type { ListResponse, Usuario } from '../../types/api'
import CourseForm, { type CourseFormValues } from './CourseForm'

type CreatedCourse = {
  id: string
}

const emptyCourse: CourseFormValues = {
  name: '',
  code: '',
  teacher_id: '',
  status: 'active',
  capacity: 1,
}

export default function CourseCreateView() {
  const { user } = useAuth()
  const navigate = useNavigate()
  const [submitError, setSubmitError] = useState<string | null>(null)
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
      const { data } = await httpClient.post<CreatedCourse>('/courses', {
        ...values,
        teacher_id: isTeacher ? user?.id : values.teacher_id,
      })
      return data
    },
  })

  async function handleSubmit(values: CourseFormValues) {
    setSubmitError(null)
    try {
      const course = await createMutation.mutateAsync(values)
      navigate(`/courses/${course.id}`, { replace: true })
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
      <CourseForm
        assignedTeacher={assignedTeacher}
        initialValues={{
          ...emptyCourse,
          teacher_id: isTeacher ? user.id : '',
        }}
        isSubmitting={createMutation.isPending}
        onSubmit={handleSubmit}
        submitError={submitError}
        submitLabel="Crear curso"
        teacherLocked={isTeacher}
        teachers={teachers}
      />
    </main>
  )
}
