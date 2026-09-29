import { useForm } from 'react-hook-form'
import type { CourseStatus, Usuario } from '../../types/api'

export type CourseFormValues = {
  name: string
  code: string
  teacher_id: string
  status: CourseStatus
  capacity: number
}

type CourseFormProps = {
  initialValues: CourseFormValues
  teachers: Usuario[]
  assignedTeacher?: Usuario
  teacherLocked: boolean
  isSubmitting: boolean
  submitError: string | null
  submitLabel: string
  onSubmit: (values: CourseFormValues) => Promise<void>
}

export default function CourseForm({
  initialValues,
  teachers,
  assignedTeacher,
  teacherLocked,
  isSubmitting,
  submitError,
  submitLabel,
  onSubmit,
}: CourseFormProps) {
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<CourseFormValues>({ defaultValues: initialValues })

  return (
    <form
      className="grid gap-5 rounded-xl bg-white p-6 shadow-sm"
      onSubmit={handleSubmit(onSubmit)}
    >
      <div className="grid gap-1">
        <label htmlFor="course-name">Nombre</label>
        <input
          className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2"
          id="course-name"
          maxLength={150}
          minLength={1}
          {...register('name', {
            required: 'El nombre es obligatorio',
            maxLength: {
              value: 150,
              message: 'El nombre no puede superar 150 caracteres',
            },
          })}
        />
        {errors.name && (
          <p className="text-sm text-red-800">{errors.name.message}</p>
        )}
      </div>

      <div className="grid gap-1">
        <label htmlFor="course-code">Código</label>
        <input
          className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2 uppercase"
          id="course-code"
          maxLength={6}
          minLength={1}
          {...register('code', {
            required: 'El código es obligatorio',
            maxLength: {
              value: 6,
              message: 'El código no puede superar 6 caracteres',
            },
          })}
        />
        {errors.code && (
          <p className="text-sm text-red-800">{errors.code.message}</p>
        )}
      </div>

      <div className="grid gap-1">
        <label htmlFor="course-teacher">Docente</label>
        {teacherLocked ? (
          <>
            <input
              className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2"
              disabled
              id="course-teacher"
              value={assignedTeacher?.name ?? ''}
            />
            <input
              type="hidden"
              {...register('teacher_id', { required: 'El docente es obligatorio' })}
            />
          </>
        ) : (
          <select
            className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2"
            id="course-teacher"
            {...register('teacher_id', {
              required: 'Selecciona un docente',
            })}
          >
            <option value="">Seleccionar docente</option>
            {teachers
              .filter((teacher) => teacher.role === 'Teacher')
              .map((teacher) => (
                <option key={teacher.id} value={teacher.id}>
                  {teacher.name} ({teacher.email})
                </option>
              ))}
          </select>
        )}
        {errors.teacher_id && (
          <p className="text-sm text-red-800">{errors.teacher_id.message}</p>
        )}
      </div>

      <div className="grid gap-1">
        <label htmlFor="course-status">Estado</label>
        <select
          className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2"
          id="course-status"
          {...register('status', { required: 'Selecciona un estado' })}
        >
          <option value="active">Activo</option>
          <option value="inactive">Inactivo</option>
        </select>
      </div>

      <div className="grid gap-1">
        <label htmlFor="course-capacity">Cupo</label>
        <input
          className="rounded-lg border border-verde-medio/30 bg-crema px-3 py-2"
          id="course-capacity"
          max={32767}
          min={1}
          type="number"
          {...register('capacity', {
            valueAsNumber: true,
            required: 'El cupo es obligatorio',
            min: { value: 1, message: 'El cupo debe ser mayor a cero' },
            max: { value: 32767, message: 'El cupo excede el máximo permitido' },
          })}
        />
        {errors.capacity && (
          <p className="text-sm text-red-800">{errors.capacity.message}</p>
        )}
      </div>

      {submitError && (
        <p className="rounded-lg bg-red-50 p-3 text-red-800" role="alert">
          {submitError}
        </p>
      )}

      <button
        className="rounded-lg bg-dorado px-4 py-3 font-semibold text-verde-oscuro disabled:opacity-60"
        disabled={isSubmitting}
        type="submit"
      >
        {isSubmitting ? 'Guardando...' : submitLabel}
      </button>
    </form>
  )
}
