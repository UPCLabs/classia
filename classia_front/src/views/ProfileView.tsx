import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'

type ProfileForm = {
  nombre: string
  correo: string
  carrera: string
  descripcion: string
  avatar?: FileList
}

const emailPattern = /^[^\s@]+@[^\s@]+\.[^\s@]+$/

export default function ProfileView() {
  const [avatarPreview, setAvatarPreview] = useState<string | null>(null)

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<ProfileForm>()

  const avatarRegister = register('avatar')

  const updateProfileMutation = useMutation({
    mutationFn: async (formData: ProfileForm) => {
      console.log(formData)
      return Promise.resolve(formData)
    },
    onSuccess: () => {
      // TODO: Mostrar feedback cuando se defina la libreria de notificaciones.
    },
  })

  const handleProfileForm = (formData: ProfileForm) => {
    updateProfileMutation.mutate(formData)
  }

  const handleCancel = () => {
    reset()
    setAvatarPreview(null)
  }

  return (
    <main className="min-h-screen bg-verde-oscuro px-4 py-10 text-verde-oscuro sm:px-6">
      <form
        className="mx-auto w-full max-w-lg rounded-2xl bg-crema p-6 shadow-lg sm:p-10"
        onSubmit={handleSubmit(handleProfileForm)}
      >
        <legend className="mb-8 text-center text-2xl font-bold">
          Editar perfil
        </legend>

        <section className="mb-8 flex flex-col items-center gap-4">
          <div className="flex h-32 w-32 items-center justify-center overflow-hidden rounded-full border-4 border-dorado bg-verde-oscuro/10">
            {avatarPreview ? (
              <img
                src={avatarPreview}
                alt="Vista previa de la foto de perfil"
                className="h-full w-full object-cover"
              />
            ) : (
              <svg
                aria-hidden="true"
                className="h-16 w-16 text-verde-oscuro/70"
                fill="none"
                viewBox="0 0 24 24"
                strokeWidth="1.5"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  d="M15.75 7.5a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.5 20.25a7.5 7.5 0 0 1 15 0"
                />
              </svg>
            )}
          </div>

          <label
            htmlFor="avatar"
            className="cursor-pointer rounded-lg border border-dorado px-5 py-2 text-sm font-semibold text-verde-oscuro transition hover:bg-dorado hover:text-verde-oscuro focus-within:ring-2 focus-within:ring-dorado focus-within:ring-offset-2 focus-within:ring-offset-crema"
          >
            Cambiar foto
          </label>
          <input
            id="avatar"
            type="file"
            accept="image/*"
            className="sr-only"
            {...avatarRegister}
            onChange={(event) => {
              avatarRegister.onChange(event)
              const file = event.target.files?.[0]
              setAvatarPreview(file ? URL.createObjectURL(file) : null)
            }}
          />
        </section>

        <div className="space-y-5">
          <div className="grid gap-2">
            <label htmlFor="nombre" className="font-semibold">
              Nombre
            </label>
            <input
              id="nombre"
              type="text"
              className="rounded-lg border border-verde-medio/20 bg-crema/80 p-3 text-verde-oscuro outline-none transition placeholder:text-verde-oscuro/50 focus:border-dorado focus:ring-2 focus:ring-dorado"
              placeholder="Tu nombre"
              {...register('nombre', {
                required: 'El nombre es obligatorio',
              })}
            />
            {errors.nombre && (
              <p className="text-sm text-red-700">{errors.nombre.message}</p>
            )}
          </div>

          <div className="grid gap-2">
            <label htmlFor="correo" className="font-semibold">
              Correo
            </label>
            <input
              id="correo"
              type="email"
              className="rounded-lg border border-verde-medio/20 bg-crema/80 p-3 text-verde-oscuro outline-none transition placeholder:text-verde-oscuro/50 focus:border-dorado focus:ring-2 focus:ring-dorado"
              placeholder="tu@correo.com"
              {...register('correo', {
                required: 'El correo es obligatorio',
                pattern: {
                  value: emailPattern,
                  message: 'Ingresa un correo valido',
                },
              })}
            />
            {errors.correo && (
              <p className="text-sm text-red-700">{errors.correo.message}</p>
            )}
          </div>

          <div className="grid gap-2">
            <label htmlFor="carrera" className="font-semibold">
              Carrera
            </label>
            <input
              id="carrera"
              type="text"
              className="rounded-lg border border-verde-medio/20 bg-crema/80 p-3 text-verde-oscuro outline-none transition placeholder:text-verde-oscuro/50 focus:border-dorado focus:ring-2 focus:ring-dorado"
              placeholder="Tu carrera"
              {...register('carrera', {
                required: 'La carrera es obligatoria',
              })}
            />
            {errors.carrera && (
              <p className="text-sm text-red-700">{errors.carrera.message}</p>
            )}
          </div>

          <div className="grid gap-2">
            <label htmlFor="descripcion" className="font-semibold">
              Descripcion
            </label>
            <textarea
              id="descripcion"
              rows={4}
              className="resize-none rounded-lg border border-verde-medio/20 bg-crema/80 p-3 text-verde-oscuro outline-none transition placeholder:text-verde-oscuro/50 focus:border-dorado focus:ring-2 focus:ring-dorado"
              placeholder="Cuentanos un poco sobre ti"
              {...register('descripcion', {
                required: 'La descripcion es obligatoria',
              })}
            />
            {errors.descripcion && (
              <p className="text-sm text-red-700">
                {errors.descripcion.message}
              </p>
            )}
          </div>
        </div>

        <div className="mt-8 grid gap-3">
          <button
            type="submit"
            disabled={updateProfileMutation.isPending}
            className="w-full rounded-lg bg-verde-medio p-3 text-lg font-bold text-crema transition hover:bg-verde-oscuro disabled:cursor-not-allowed disabled:opacity-70"
          >
            {updateProfileMutation.isPending ? 'Guardando...' : 'Guardar cambios'}
          </button>

          <button
            type="button"
            className="w-full rounded-lg border border-dorado p-3 text-lg font-bold text-verde-oscuro transition hover:bg-dorado/20"
            onClick={handleCancel}
          >
            Cancelar
          </button>
        </div>
      </form>
    </main>
  )
}
