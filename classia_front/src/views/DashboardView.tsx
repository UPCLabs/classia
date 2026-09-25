import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useAuth } from '../auth/context'
import getErrorMessage from '../auth/getErrorMessage'
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
  const { user, signOut } = useAuth()
  const navigate = useNavigate()
  const [isSigningOut, setIsSigningOut] = useState(false)
  const [error, setError] = useState<string | null>(null)

  if (!user) {
    return null
  }

  async function handleSignOut() {
    setIsSigningOut(true)
    setError(null)

    try {
      await signOut()
      navigate('/auth/login', { replace: true })
    } catch (signOutError: unknown) {
      setError(getErrorMessage(signOutError))
    } finally {
      setIsSigningOut(false)
    }
  }

  const canViewCourses = user.role === 'Teacher' || user.role === 'Student'
  const canManageUsers = user.role === 'SuperAdmin' || user.role === 'Admin'

  return (
    <main className="min-h-screen bg-crema text-verde-oscuro">
      <header className="flex flex-col gap-4 border-b-2 border-dorado bg-verde-oscuro px-6 py-5 text-crema sm:flex-row sm:items-center sm:justify-between">
        <Link className="text-xl font-bold" to="/dashboard">
          Classia
        </Link>
        <button
          className="self-start rounded-lg border border-dorado px-4 py-2 font-semibold text-crema transition hover:bg-dorado hover:text-verde-oscuro disabled:opacity-60 sm:self-auto"
          disabled={isSigningOut}
          onClick={handleSignOut}
          type="button"
        >
          {isSigningOut ? 'Cerrando sesión...' : 'Cerrar sesión'}
        </button>
      </header>

      <section className="mx-auto max-w-5xl px-4 py-12 sm:px-6">
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

        {error && (
          <p className="mt-6 rounded-lg border border-red-700 bg-white p-4 text-red-800" role="alert">
            {error}
          </p>
        )}

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
      </section>
    </main>
  )
}
