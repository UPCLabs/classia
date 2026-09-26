import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import httpClient from '../api/httpClient'
import getErrorMessage from '../auth/getErrorMessage'
import type { UserRole } from '../types/api'

export default function RegisterView() {
  const navigate = useNavigate()
  const [name, setName] = useState('')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [role, setRole] = useState<UserRole>('Student')
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function handleSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setIsSubmitting(true)
    setError(null)

    try {
      await httpClient.post('/users/create', { name, email, password, role })
      navigate('/admin/users', {
        replace: true,
        state: { message: 'El usuario se creó correctamente.' },
      })
    } catch (requestError: unknown) {
      setError(getErrorMessage(requestError))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <main className="flex min-h-full items-center justify-center px-4 py-10">
      <div className="w-full max-w-lg rounded-2xl bg-white p-6 shadow-lg sm:p-8">
        <h1 className="mb-2 text-2xl font-bold text-verde-oscuro">
          Crear usuario
        </h1>
        <p className="mb-6 text-verde-oscuro/70">
          Registra un estudiante, docente o administrador en Classia.
        </p>

        <form className="flex flex-col gap-4" onSubmit={handleSubmit}>
          <div className="flex flex-col gap-1">
            <label className="text-sm font-medium" htmlFor="register-name">
              Nombre completo
            </label>
            <input
              autoComplete="name"
              className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2 text-verde-oscuro outline-none focus:ring-2 focus:ring-dorado"
              id="register-name"
              maxLength={100}
              minLength={1}
              onChange={(event) => setName(event.target.value)}
              required
              type="text"
              value={name}
            />
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-sm font-medium" htmlFor="register-email">
              Correo electrónico
            </label>
            <input
              autoComplete="email"
              className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2 text-verde-oscuro outline-none focus:ring-2 focus:ring-dorado"
              id="register-email"
              onChange={(event) => setEmail(event.target.value)}
              required
              type="email"
              value={email}
            />
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-sm font-medium" htmlFor="register-password">
              Contraseña inicial
            </label>
            <input
              autoComplete="new-password"
              className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2 text-verde-oscuro outline-none focus:ring-2 focus:ring-dorado"
              id="register-password"
              maxLength={20}
              minLength={8}
              onChange={(event) => setPassword(event.target.value)}
              required
              type="password"
              value={password}
            />
            <span className="text-xs text-verde-oscuro/70">
              Debe tener entre 8 y 20 caracteres.
            </span>
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-sm font-medium" htmlFor="register-role">
              Rol
            </label>
            <select
              className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2 text-verde-oscuro outline-none focus:ring-2 focus:ring-dorado"
              id="register-role"
              onChange={(event) => setRole(event.target.value as UserRole)}
              value={role}
            >
              <option value="Student">Estudiante</option>
              <option value="Teacher">Docente</option>
              <option value="Admin">Administrador</option>
              <option value="SuperAdmin">Superadministrador</option>
            </select>
          </div>

          {error && (
            <p className="rounded-lg bg-red-50 px-3 py-2 text-sm text-red-800" role="alert">
              {error}
            </p>
          )}

          <div className="mt-2 flex flex-col-reverse gap-3 sm:flex-row sm:justify-end">
            <Link
              className="rounded-lg border border-verde-medio px-4 py-2 text-center font-semibold text-verde-oscuro"
              to="/admin/users"
            >
              Cancelar
            </Link>
            <button
              className="rounded-lg bg-dorado px-4 py-2 font-semibold text-verde-oscuro hover:brightness-110 disabled:opacity-60"
              disabled={isSubmitting}
              type="submit"
            >
              {isSubmitting ? 'Creando...' : 'Crear usuario'}
            </button>
          </div>
        </form>
      </div>
    </main>
  )
}
