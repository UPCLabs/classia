import { useForm, SubmitHandler } from "react-hook-form";

interface CourseFormData {
  name: string;
  code: number;
  teacher_id: number;
  status: string;
  capacity: number;
}

interface CourseFormProps {
  mode: 'create' | 'edit';
  defaultValues?: CourseFormData;
}

export default function CourseForm({ mode, defaultValues }: CourseFormProps) {
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<CourseFormData>({ defaultValues });

  const onSubmit: SubmitHandler<CourseFormData> = (data) => {
    console.log("Datos enviados:", data);
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-[#123F36]">
      <div className="bg-[#2A6B5C] rounded-2xl shadow-xl p-8 w-full max-w-sm">
        <h1 className="text-[#E8DCC4] text-2xl font-bold mb-6 text-center">
          {mode === 'create' ? 'Crear curso' : 'Editar curso'}
        </h1>

        <form onSubmit={handleSubmit(onSubmit)} className="flex flex-col gap-4">
          <div className="flex flex-col gap-1">
            <label className="text-[#E8DCC4] text-sm font-medium">Nombre</label>
            <input
              type="text"
              {...register("name", { required: "El nombre es obligatorio" })}
              className="bg-[#E8DCC4] text-[#123F36] rounded-lg px-3 py-2 outline-none focus:ring-2 focus:ring-[#C49A45]"
            />
            {errors.name && <span className="text-red-300 text-xs">{errors.name.message}</span>}
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-[#E8DCC4] text-sm font-medium">Código</label>
            <input
              type="number"
              {...register("code", { valueAsNumber: true, required: "El código es obligatorio" })}
              className="bg-[#E8DCC4] text-[#123F36] rounded-lg px-3 py-2 outline-none focus:ring-2 focus:ring-[#C49A45]"
            />
            {errors.code && <span className="text-red-300 text-xs">{errors.code.message}</span>}
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-[#E8DCC4] text-sm font-medium">ID del profesor</label>
            <input
              type="number"
              {...register("teacher_id", { valueAsNumber: true, required: "El ID del profesor es obligatorio" })}
              className="bg-[#E8DCC4] text-[#123F36] rounded-lg px-3 py-2 outline-none focus:ring-2 focus:ring-[#C49A45]"
            />
            {errors.teacher_id && <span className="text-red-300 text-xs">{errors.teacher_id.message}</span>}
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-[#E8DCC4] text-sm font-medium">Estado</label>
            <input
              type="text"
              {...register("status", { required: "El estado es obligatorio" })}
              className="bg-[#E8DCC4] text-[#123F36] rounded-lg px-3 py-2 outline-none focus:ring-2 focus:ring-[#C49A45]"
            />
            {errors.status && <span className="text-red-300 text-xs">{errors.status.message}</span>}
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-[#E8DCC4] text-sm font-medium">Capacidad</label>
            <input
              type="number"
              {...register("capacity", {
                valueAsNumber: true,
                required: "La capacidad es obligatoria",
                min: { value: 1, message: "Debe ser mayor a 0" },
              })}
              className="bg-[#E8DCC4] text-[#123F36] rounded-lg px-3 py-2 outline-none focus:ring-2 focus:ring-[#C49A45]"
            />
            {errors.capacity && <span className="text-red-300 text-xs">{errors.capacity.message}</span>}
          </div>

          <button type="submit" className="bg-[#C49A45] text-[#123F36] font-semibold rounded-lg py-2 mt-2 hover:brightness-110 transition">
            {mode === 'create' ? 'Crear curso' : 'Guardar cambios'}
          </button>

          <button type="button" className="text-[#E8DCC4] text-sm underline hover:text-[#C49A45] transition">
            Cancelar
          </button>
        </form>
      </div>
    </div>
  );
}