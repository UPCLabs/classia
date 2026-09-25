import { render, screen } from '@testing-library/react'
import CoursesView from '../src/views/StudentCourses/CoursesView'

describe('CoursesView', () => {
  it('loads and renders the static student and courses', async () => {
    render(<CoursesView />)

    expect(screen.getByText('Cargando...')).toBeInTheDocument()
    expect(await screen.findByText('Juan Pérez')).toBeInTheDocument()
    expect(screen.getByText('A00123456')).toBeInTheDocument()
    expect(screen.getByText('Programación Orientada a Objetos')).toBeInTheDocument()
    expect(screen.getByText('Bases de Datos I')).toBeInTheDocument()
    expect(screen.getByText('Cálculo Integral')).toBeInTheDocument()
  })
})
