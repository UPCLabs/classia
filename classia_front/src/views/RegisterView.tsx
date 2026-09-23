import { useState } from 'react'

export default function RegisterView() {
    const [code, setCode] = useState('')
    const [name, setName] = useState('')
    const [email, setEmail] = useState('')
    const [career, setCareer] = useState('')

    const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
        e.preventDefault()
        console.log('Register with:', { code, name, email, career })
    }

    return (
        <div className="min-h-screen flex items-center justify-center bg-[#123F36]">
            <div className="bg-[#2A6B5C] rounded-2xl shadow-xl p-8 w-full max-w-sm">
                <h1 className="text-[#E8DCC4] text-2xl font-bold mb-6 text-center">
                    Registro de usuario
                </h1>

                <form onSubmit={handleSubmit} className="flex flex-col gap-4">
                    <div className="flex flex-col gap-1">
                        <label htmlFor="register-name" className="text-[#E8DCC4] text-sm font-medium">Nombre completo</label>
                        <input
                            id="register-name"
                            type="text"
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                            className="bg-[#E8DCC4] text-[#123F36] rounded-lg px-3 py-2 outline-none focus:ring-2 focus:ring-[#C49A45]"
                        />
                    </div>

                    <div className="flex flex-col gap-1">
                        <label htmlFor="register-email" className="text-[#E8DCC4] text-sm font-medium">Correo</label>
                        <input
                            id="register-email"
                            type="text"
                            value={email}
                            onChange={(e) => setEmail(e.target.value)}
                            className="bg-[#E8DCC4] text-[#123F36] rounded-lg px-3 py-2 outline-none focus:ring-2 focus:ring-[#C49A45]"
                        />
                    </div>

                    <div className="flex flex-col gap-1">
                        <label htmlFor="register-code" className="text-[#E8DCC4] text-sm font-medium">Código</label>
                        <input
                            id="register-code"
                            type="text"
                            value={code}
                            onChange={(e) => setCode(e.target.value)}
                            className="bg-[#E8DCC4] text-[#123F36] rounded-lg px-3 py-2 outline-none focus:ring-2 focus:ring-[#C49A45]"
                        />
                    </div>

                    <div className="flex flex-col gap-1">
                        <label htmlFor="register-career" className="text-[#E8DCC4] text-sm font-medium">Carrera</label>
                        <input
                            id="register-career"
                            type="text"
                            value={career}
                            onChange={(e) => setCareer(e.target.value)}
                            className="bg-[#E8DCC4] text-[#123F36] rounded-lg px-3 py-2 outline-none focus:ring-2 focus:ring-[#C49A45]"
                        />
                    </div>

                    <button
                        type="submit"
                        className="bg-[#C49A45] text-[#123F36] font-semibold rounded-lg py-2 mt-2 hover:brightness-110 transition"
                    >
                        Registrarse
                    </button>
                </form>
            </div>
        </div>
    )
}
