import { Link } from 'react-router-dom'
import { useAuth } from '../auth/context'
import type { UserRole } from '../types/api'

const roleNames: Record<UserRole, string> = {
  SuperAdmin: 'Superadministrador',
  Admin: 'Administrador',
  Teacher: 'Docente',
  Student: 'Estudiante',
}

const roleDescriptions: Record<UserRole, string> = {
  SuperAdmin: 'Administra la plataforma y sus usuarios.',
  Admin: 'Administra usuarios y organiza la comunidad académica.',
  Teacher: 'Consulta tus cursos y acompaña a tus estudiantes.',
  Student: 'Consulta tus cursos y mantente al día con tu aprendizaje.',
}

export default function DashboardView() {
  const { user } = useAuth()

  if (!user) {
    return null
  }

  const canViewCourses = user.role === 'Teacher' || user.role === 'Student'
  const canManageUsers = user.role === 'SuperAdmin' || user.role === 'Admin'

  return (
    <main className="mx-auto max-w-5xl px-4 py-12 sm:px-6">
      <p className="text-sm font-semibold uppercase tracking-[0.2em] text-verde-medio">
        {roleNames[user.role]}
      </p>
      <h1 className="mt-2 text-3xl font-bold sm:text-4xl">
        Panel principal
      </h1>
      <p className="mt-3 text-lg">
        Hola, {user.name}. {roleDescriptions[user.role]}
      </p>
      <p className="mt-1 text-verde-oscuro/70">{user.email}</p>

      <div className="mt-9 grid gap-5 sm:grid-cols-2">
        {canViewCourses && (
          <Link
            className="rounded-xl border-l-4 border-dorado bg-white p-6 shadow-sm transition hover:shadow-md"
            to="/courses"
          >
            <h2 className="text-xl font-bold">Mis cursos</h2>
            <p className="mt-2 text-verde-oscuro/75">
              Consulta los cursos asociados a tu cuenta.
            </p>
          </Link>
        )}

        {canManageUsers && (
          <section className="rounded-xl border-l-4 border-dorado bg-white p-6 shadow-sm">
            <h2 className="text-xl font-bold">Administración</h2>
            <p className="mt-2 text-verde-oscuro/75">
              La gestión de usuarios estará disponible próximamente.
            </p>
          </section>
        )}
      </div>
    </main>
  )
}
