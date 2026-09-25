import { useForm } from 'react-hook-form'

interface ChangePasswordForm {
  currentPassword: string
  newPassword: string
  confirmationPassword: string
}

export default function ChangePasswordView() {
  const {
    register,
    handleSubmit,
    watch,
    formState: { errors },
  } = useForm<ChangePasswordForm>()

  const onSubmit = (data: ChangePasswordForm) => {
    console.log(data)
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-[#123F36]">
      <div className="bg-[#2A6B5C] rounded-2xl shadow-xl p-8 w-full max-w-sm">
        <h1 className="text-[#E8DCC4] text-2xl font-bold mb-6 text-center">
          Cambiar contraseña
        </h1>

        <form onSubmit={handleSubmit(onSubmit)} className="flex flex-col gap-4">
          <div className="flex flex-col gap-1">
            <label className="text-[#E8DCC4] text-sm font-medium">
              Contraseña actual
            </label>
            <input
              type="password"
              {...register('currentPassword', {
                required: 'Este campo es obligatorio',
              })}
              className="bg-[#E8DCC4] text-[#123F36] rounded-lg px-3 py-2 outline-none focus:ring-2 focus:ring-[#C49A45]"
            />
            {errors.currentPassword && (
              <span className="text-red-300 text-xs">
                {errors.currentPassword.message}
              </span>
            )}
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-[#E8DCC4] text-sm font-medium">
              Nueva contraseña
            </label>
            <input
              type="password"
              {...register('newPassword', {
                required: 'Este campo es obligatorio',
              })}
              className="bg-[#E8DCC4] text-[#123F36] rounded-lg px-3 py-2 outline-none focus:ring-2 focus:ring-[#C49A45]"
            />
            {errors.newPassword && (
              <span className="text-red-300 text-xs">
                {errors.newPassword.message}
              </span>
            )}
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-[#E8DCC4] text-sm font-medium">
              Confirmar contraseña
            </label>
            <input
              type="password"
              {...register('confirmationPassword', {
                required: 'Este campo es obligatorio',
                validate: (value) =>
                  value === watch('newPassword') || 'Las contraseñas no coinciden',
              })}
              className="bg-[#E8DCC4] text-[#123F36] rounded-lg px-3 py-2 outline-none focus:ring-2 focus:ring-[#C49A45]"
            />
            {errors.confirmationPassword && (
              <span className="text-red-300 text-xs">
                {errors.confirmationPassword.message}
              </span>
            )}
          </div>

          <button
            type="submit"
            className="bg-[#C49A45] text-[#123F36] font-semibold rounded-lg py-2 mt-2 hover:brightness-110 transition"
          >
            Guardar cambios
          </button>

          <button
            type="button"
            className="text-[#E8DCC4] text-sm underline hover:text-[#C49A45] transition"
          >
            Cancelar
          </button>
        </form>
      </div>
    </div>
  )
}