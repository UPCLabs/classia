export type UserRole = 'SuperAdmin' | 'Admin' | 'Teacher' | 'Student'

export type UserStatus = 'active' | 'inactive'

export type CourseStatus = 'active' | 'inactive'

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

export type ListResponse<T> = {
  items: T[]
}

export type ApiError = {
  code: string
  message: string
}
