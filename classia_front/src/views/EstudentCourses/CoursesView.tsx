import { useState, useEffect } from 'react'

interface Course {
  id: number
  name: string
}

interface Student {
  name: string
  code: string
}

export default function CoursesView() {

  const [student, setStudent] = useState<Student | null>(null)

  useEffect(() => {

    setStudent({
      name: 'Juan Pérez',
      code: 'A00123456',
    })
  }, [])


  const courses: Course[] = [
    { id: 1, name: 'Programación Orientada a Objetos' },
    { id: 2, name: 'Bases de Datos I' },
    { id: 3, name: 'Cálculo Integral' },
  ]

 if (!student) {
  return (
    <div className="min-h-screen flex items-center justify-center bg-[#123F36]">
      <p className="text-[#E8DCC4]">Cargando...</p>
    </div>
  )
}

return (
  <div className="min-h-screen bg-[#123F36]">
    <header className="bg-[#2A6B5C] px-6 py-4 flex justify-between items-center shadow-md border-b-2 border-[#C49A45]">
      <h1 className="text-[#E8DCC4] font-semibold text-lg">
        {student.name}
      </h1>
      <span className="text-[#C49A45] text-sm font-medium">
        {student.code}
      </span>
    </header>

    <div className="flex justify-center py-10 px-4">
      <div className="w-full max-w-md">
        <h2 className="text-[#E8DCC4] text-xl font-bold mb-4 border-l-4 border-[#C49A45] pl-3">
          Mis cursos
        </h2>

        <div className="flex flex-col gap-3">
          {courses.map((course) => (
            <div
              key={course.id}
              className="bg-[#2A6B5C] text-[#E8DCC4] rounded-lg px-4 py-3 shadow border-l-4 border-[#C49A45] hover:border-[#E8DCC4] transition"
            >
              {course.name}
            </div>
          ))}
        </div>
      </div>
    </div>
  </div>
)
}