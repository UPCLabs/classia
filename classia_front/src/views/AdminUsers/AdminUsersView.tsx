import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link, useLocation } from 'react-router-dom'
import httpClient from '../../api/httpClient'
import getErrorMessage from '../../auth/getErrorMessage'
import AsyncStateView from '../../shared/components/AsyncStateView'
import { useAsyncState } from '../../shared/hooks/useAsyncState'
import type { ListResponse, UserRole, UserStatus, Usuario } from '../../types/api'

type AdminUsersLocationState = {
  message?: string
}

const userRoles: UserRole[] = ['SuperAdmin', 'Admin', 'Teacher', 'Student']

export default function AdminUsersView() {
  const [query, setQuery] = useState('')
  const [role, setRole] = useState<UserRole | ''>('')
  const [status, setStatus] = useState<UserStatus | ''>('')
  const queryClient = useQueryClient()
  const location = useLocation()
  const locationState = location.state as AdminUsersLocationState | null
  const params = new URLSearchParams()

  if (query.trim()) params.set('q', query.trim())
  if (role) params.set('role', role)
  if (status) params.set('status', status)

  const usersQuery = useQuery({
    queryKey: ['admin-users', query.trim(), role, status],
    queryFn: async () => {
      const search = params.toString()
      const { data } = await httpClient.get<ListResponse<Usuario>>(
        `/users${search ? `?${search}` : ''}`,
      )
      return data.items
    },
  })

  const statusMutation = useMutation({
    mutationFn: async ({ id, status: nextStatus }: { id: string; status: UserStatus }) => {
      const { data } = await httpClient.patch<Usuario>(`/users/${id}/status`, {
        status: nextStatus,
      })
      return data
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ['admin-users'] }),
  })

  const usersState = useAsyncState({
    data: usersQuery.data,
    isLoading: usersQuery.isLoading,
    error: usersQuery.error
      ? { code: 'REQUEST_ERROR', message: getErrorMessage(usersQuery.error) }
      : null,
  })

  return (
    <main className="mx-auto max-w-6xl px-4 py-10 sm:px-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <p className="text-sm font-semibold uppercase tracking-[0.2em] text-verde-medio">
            Administración
          </p>
          <h1 className="mt-2 text-3xl font-bold">Usuarios</h1>
          <p className="mt-2 text-verde-oscuro/70">
            Busca usuarios, actualiza sus datos y administra su acceso.
          </p>
        </div>
        <Link
          className="rounded-lg bg-dorado px-5 py-3 text-center font-semibold text-verde-oscuro transition hover:brightness-110"
          to="/admin/users/new"
        >
          Crear usuario
        </Link>
      </div>

      {locationState?.message && (
        <p className="mt-6 rounded-lg border border-verde-medio bg-white p-4 text-verde-oscuro" role="status">
          {locationState.message}
        </p>
      )}

      {statusMutation.error && (
        <p className="mt-6 rounded-lg bg-red-50 p-4 text-red-800" role="alert">
          {getErrorMessage(statusMutation.error)}
        </p>
      )}

      <section aria-label="Filtros de usuarios" className="my-7 grid gap-4 rounded-xl bg-white p-4 shadow-sm sm:grid-cols-3">
        <div className="grid gap-1">
          <label className="text-sm font-medium" htmlFor="users-search">
            Buscar
          </label>
          <input
            className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2 outline-none focus:ring-2 focus:ring-dorado"
            id="users-search"
            onChange={(event) => setQuery(event.target.value)}
            placeholder="Nombre o correo"
            type="search"
            value={query}
          />
        </div>
        <div className="grid gap-1">
          <label className="text-sm font-medium" htmlFor="users-role">
            Rol
          </label>
          <select
            className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2 outline-none focus:ring-2 focus:ring-dorado"
            id="users-role"
            onChange={(event) => setRole(event.target.value as UserRole | '')}
            value={role}
          >
            <option value="">Todos</option>
            {userRoles.map((userRole) => (
              <option key={userRole} value={userRole}>
                {userRole}
              </option>
            ))}
          </select>
        </div>
        <div className="grid gap-1">
          <label className="text-sm font-medium" htmlFor="users-status">
            Estado
          </label>
          <select
            className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2 outline-none focus:ring-2 focus:ring-dorado"
            id="users-status"
            onChange={(event) => setStatus(event.target.value as UserStatus | '')}
            value={status}
          >
            <option value="">Todos</option>
            <option value="active">Activo</option>
            <option value="inactive">Inactivo</option>
          </select>
        </div>
      </section>

      <AsyncStateView
        error={usersState.error}
        isEmpty={usersState.isEmpty}
        isLoading={usersState.isLoading}
      >
        <div className="overflow-x-auto rounded-xl bg-white shadow-sm">
          <table className="w-full min-w-[680px] border-collapse text-left">
            <thead className="bg-verde-oscuro text-crema">
              <tr>
                <th className="px-4 py-3">Usuario</th>
                <th className="px-4 py-3">Rol</th>
                <th className="px-4 py-3">Estado</th>
                <th className="px-4 py-3">Acciones</th>
              </tr>
            </thead>
            <tbody>
              {usersState.data?.map((user) => (
                <tr className="border-b border-verde-oscuro/10 last:border-0" key={user.id}>
                  <td className="px-4 py-4">
                    <p className="font-semibold">{user.name}</p>
                    <p className="text-sm text-verde-oscuro/70">{user.email}</p>
                  </td>
                  <td className="px-4 py-4">{user.role}</td>
                  <td className="px-4 py-4">
                    <span className={user.status === 'active' ? 'text-green-800' : 'text-red-800'}>
                      {user.status === 'active' ? 'Activo' : 'Inactivo'}
                    </span>
                  </td>
                  <td className="px-4 py-4">
                    <div className="flex items-center gap-4">
                      <Link
                        className="font-semibold text-verde-medio underline"
                        to={`/admin/users/${user.id}/edit`}
                      >
                        Editar
                      </Link>
                      <button
                        className="font-semibold text-verde-oscuro underline disabled:opacity-50"
                        disabled={statusMutation.isPending}
                        onClick={() =>
                          statusMutation.mutate({
                            id: user.id,
                            status: user.status === 'active' ? 'inactive' : 'active',
                          })
                        }
                        type="button"
                      >
                        {user.status === 'active' ? 'Desactivar' : 'Activar'}
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </AsyncStateView>
    </main>
  )
}
