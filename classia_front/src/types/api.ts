export type UserRole = 'SuperAdmin' | 'Admin' | 'Teacher' | 'Student'

export type UserStatus = 'active' | 'inactive'

export type CourseStatus = 'active' | 'inactive'

export type ActivityStatus = 'draft' | 'published'

export type Usuario = {
  id: string
  name: string
  email: string
  role: UserRole
  status: UserStatus
  created_at: string
  updated_at: string
}

export type Curso = {
  id: string
  name: string
  code: string
  teacher: {
    id: string
    name: string
    email: string
  }
  status: CourseStatus
  capacity: number
  enrolled_count: number
  created_at: string
  updated_at: string
}

export type Activity = {
  id: string
  course_id: string
  title: string
  description: string
  due_at: string
  status: ActivityStatus
  published_at: string | null
  created_at: string
  updated_at: string
}

export type ActivityCreate = Pick<
  Activity,
  'title' | 'description' | 'due_at'
>

export type ActivityUpdate = Partial<ActivityCreate>

export type Inscripcion = {
  course_id: string
  student: Pick<Usuario, 'id' | 'name' | 'email' | 'role' | 'status'>
  enrolled_at: string
}

export type ListResponse<T> = {
  items: T[]
}

export type DeliveryStatus = 'submitted' | 'replaced' | 'graded'

export type Evaluacion = {
  score: number
  feedback: string | null
  graded_at: string
  updated_at: string
}

export type Entrega = {
  id: string
  activity_id: string
  student: {
    id: string
    name: string
    email: string
  }
  status: DeliveryStatus
  version: number
  submitted_at: string
  updated_at: string
  evaluation: Evaluacion | null
}

export type EntregaPropia = {
  delivery: Entrega | null
}

export type EntregasRecibidas = {
  items: Entrega[]
  total: number
}

export type CalificacionInput = {
  score: number
  feedback?: string | null
  expected_version: number
}

export type MfaSetup = {
  secret: string
  url: string
}

export type MfaChallenge = {
  verified: boolean
}

export type ApiError = {
  code: string
  message: string
}

export type AuthUser = {
  id: string
  name: string
  email: string
  role: UserRole
}
