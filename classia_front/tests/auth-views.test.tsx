import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { vi } from 'vitest'
import LoginView from '../src/views/LoginView'
import RegisterView from '../src/views/RegisterView'

describe('LoginView', () => {
  it('edits the current fields and submits their values', async () => {
    const user = userEvent.setup()
    const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => undefined)
    render(<LoginView />)

    await user.type(screen.getByLabelText('Usuario'), 'student')
    await user.type(screen.getByLabelText('Contraseña'), 'secret')
    await user.click(screen.getByRole('button', { name: 'Log in' }))

    expect(consoleSpy).toHaveBeenCalledWith('Login con:', {
      user: 'student',
      password: 'secret',
    })
    consoleSpy.mockRestore()
  })
})

describe('RegisterView', () => {
  it('edits and submits the registration fields', async () => {
    const user = userEvent.setup()
    const consoleSpy = vi.spyOn(console, 'log').mockImplementation(() => undefined)
    render(<RegisterView />)

    await user.type(screen.getByLabelText('Nombre completo'), 'Ana Torres')
    await user.type(screen.getByLabelText('Correo'), 'ana@example.com')
    await user.type(screen.getByLabelText('Código'), 'A001')
    await user.type(screen.getByLabelText('Carrera'), 'Sistemas')
    await user.click(screen.getByRole('button', { name: 'Registrarse' }))

    expect(consoleSpy).toHaveBeenCalledWith('Register with:', {
      code: 'A001',
      name: 'Ana Torres',
      email: 'ana@example.com',
      career: 'Sistemas',
    })
    consoleSpy.mockRestore()
  })
})
