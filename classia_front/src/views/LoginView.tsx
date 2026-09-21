import { useState } from 'react'
export default function LoginView() {

  const [usuario, setUsuario] = useState('')
  const [contraseña, setContraseña] = useState('')

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    console.log('Login con:', { usuario, contraseña })
  }

  return (
     <div className="min-h-screen flex items-center justify-center bg-[#123F36]">
      <div className="bg-[#2A6B5C] rounded-2xl shadow-xl p-8 w-full max-w-sm">
        <h1 className="text-[#E8DCC4] text-2xl font-bold mb-6 text-center">
          Iniciar sesión
        </h1>

        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <div className="flex flex-col gap-1">
            <label className="text-[#E8DCC4] text-sm font-medium">Usuario</label>
            <input
              type="text"
              value={usuario}
              onChange={(e) => setUsuario(e.target.value)}
              className="bg-[#E8DCC4] text-[#123F36] rounded-lg px-3 py-2 outline-none focus:ring-2 focus:ring-[#C49A45]"
            />
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-[#E8DCC4] text-sm font-medium">Contraseña</label>
            <input
              type="password"
              value={contraseña}
              onChange={(e) => setContraseña(e.target.value)}
              className="bg-[#E8DCC4] text-[#123F36] rounded-lg px-3 py-2 outline-none focus:ring-2 focus:ring-[#C49A45]"
            />
          </div>

          <button
            type="submit"
            className="bg-[#C49A45] text-[#123F36] font-semibold rounded-lg py-2 mt-2 hover:brightness-110 transition"
          >
            Log in
          </button>

          <button
            type="button"
            className="text-[#E8DCC4] text-sm underline hover:text-[#C49A45] transition"
          >
            Registrarse
          </button>

          <button
            type="button"
            className="text-[#E8DCC4] text-sm underline hover:text-[#C49A45] transition"
          >
            Olvidé mi contraseña
          </button>
        </form>
      </div>
    </div>
  )
}
