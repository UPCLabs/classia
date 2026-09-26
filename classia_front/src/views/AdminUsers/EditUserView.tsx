import { useEffect } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import { Link, useNavigate, useParams } from 'react-router-dom'
import httpClient from '../../api/httpClient'
import getErrorMessage from '../../auth/getErrorMessage'
import type { UserRole, Usuario } from '../../types/api'

type EditUserForm = {
  name: string
  email: string
  role: UserRole
}

type UpdateUserPayload = {
  name?: string
  email?: string
  role?: UserRole
}

export default function EditUserView() {
  const { userId } = useParams()
  const navigate = useNavigate()
  const queryClient = useQueryClient()
  const userQuery = useQuery({
    queryKey: ['admin-user', userId],
    enabled: Boolean(userId),
    queryFn: async () => {
      const { data } = await httpClient.get<Usuario>(`/users/${userId}`)
      return data
    },
  })
  const {
    register,
    handleSubmit,
    reset,
    setError,
    clearErrors,
    formState: { errors, isSubmitting },
  } = useForm<EditUserForm>()

  useEffect(() => {
    if (userQuery.data) {
      reset({
        name: userQuery.data.name,
        email: userQuery.data.email,
        role: userQuery.data.role,
      })
    }
  }, [reset, userQuery.data])

  const updateMutation = useMutation({
    mutationFn: async (payload: UpdateUserPayload) => {
      const { data } = await httpClient.patch<Usuario>(
        `/users/${userId}`,
        payload,
      )
      return data
    },
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ['admin-users'] })
      navigate('/admin/users', {
        replace: true,
        state: { message: 'Los datos del usuario se actualizaron.' },
      })
    },
  })

  async function onSubmit(values: EditUserForm) {
    const currentUser = userQuery.data
    if (!currentUser) return

    const payload: UpdateUserPayload = {}
    if (values.name !== currentUser.name) payload.name = values.name
    if (values.email !== currentUser.email) payload.email = values.email
    if (values.role !== currentUser.role) payload.role = values.role

    if (Object.keys(payload).length === 0) {
      setError('root', {
        type: 'manual',
        message: 'Modifica al menos un campo antes de guardar.',
      })
      return
    }

    clearErrors('root')
    await updateMutation.mutateAsync(payload)
  }

  if (userQuery.isLoading) {
    return <p className="p-10 text-center" role="status">Cargando usuario...</p>
  }

  if (userQuery.error) {
    return (
      <main className="mx-auto max-w-xl px-4 py-10">
        <p className="rounded-lg bg-red-50 p-4 text-red-800" role="alert">
          {getErrorMessage(userQuery.error)}
        </p>
        <Link className="mt-4 inline-block underline" to="/admin/users">
          Volver a usuarios
        </Link>
      </main>
    )
  }

  return (
    <main className="mx-auto max-w-xl px-4 py-10">
      <h1 className="mb-6 text-3xl font-bold">Editar usuario</h1>
      {updateMutation.error && (
        <p className="mb-4 rounded-lg bg-red-50 p-4 text-red-800" role="alert">
          {getErrorMessage(updateMutation.error)}
        </p>
      )}

      <form
        className="grid gap-5 rounded-xl bg-white p-6 shadow-sm"
        onSubmit={handleSubmit(onSubmit)}
      >
        {errors.root && (
          <p className="rounded-lg bg-amber-50 p-3 text-amber-900" role="alert">
            {errors.root.message}
          </p>
        )}
        <div className="grid gap-1">
          <label htmlFor="edit-user-name">Nombre</label>
          <input
            className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2"
            id="edit-user-name"
            maxLength={100}
            minLength={1}
            {...register('name', { required: 'El nombre es obligatorio' })}
          />
          {errors.name && <p className="text-sm text-red-800">{errors.name.message}</p>}
        </div>

        <div className="grid gap-1">
          <label htmlFor="edit-user-email">Correo</label>
          <input
            className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2"
            id="edit-user-email"
            type="email"
            {...register('email', {
              required: 'El correo es obligatorio',
              pattern: {
                value: /^[^\s@]+@[^\s@]+\.[^\s@]+$/,
                message: 'Ingresa un correo válido',
              },
            })}
          />
          {errors.email && <p className="text-sm text-red-800">{errors.email.message}</p>}
        </div>

        <div className="grid gap-1">
          <label htmlFor="edit-user-role">Rol</label>
          <select
            className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2"
            id="edit-user-role"
            {...register('role', { required: true })}
          >
            <option value="SuperAdmin">Superadministrador</option>
            <option value="Admin">Administrador</option>
            <option value="Teacher">Docente</option>
            <option value="Student">Estudiante</option>
          </select>
        </div>

        <p className="text-sm text-verde-oscuro/70">
          Solo se enviarán los campos que cambies.
        </p>

        <div className="flex flex-col-reverse gap-3 sm:flex-row sm:justify-end">
          <Link
            className="rounded-lg border border-verde-medio px-4 py-2 text-center font-semibold"
            to="/admin/users"
          >
            Cancelar
          </Link>
          <button
            className="rounded-lg bg-dorado px-4 py-2 font-semibold disabled:opacity-60"
            disabled={isSubmitting || updateMutation.isPending}
            type="submit"
          >
            {isSubmitting || updateMutation.isPending ? 'Guardando...' : 'Guardar cambios'}
          </button>
        </div>
      </form>
    </main>
  )
}
