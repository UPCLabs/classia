import { Link } from 'react-router-dom'

const features = [
  {
    icon: (
      <svg
        aria-hidden="true"
        className="h-8 w-8"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        strokeWidth="1.8"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          d="M4.5 5.25A2.25 2.25 0 0 1 6.75 3h10.5a2.25 2.25 0 0 1 2.25 2.25v13.5A2.25 2.25 0 0 1 17.25 21H6.75a2.25 2.25 0 0 1-2.25-2.25V5.25Z"
        />
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          d="M8 7.5h8M8 11.25h8M8 15h4"
        />
      </svg>
    ),
    title: 'Gestión de cursos',
    description:
      'Organiza la experiencia académica en un solo lugar: crea, consulta y actualiza tus cursos con claridad.',
  },
  {
    icon: (
      <svg
        aria-hidden="true"
        className="h-8 w-8"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        strokeWidth="1.8"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          d="M12 6.75v10.5M6.75 12h10.5"
        />
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          d="M5.25 3.75h13.5A1.5 1.5 0 0 1 20.25 5.25v13.5a1.5 1.5 0 0 1-1.5 1.5H5.25a1.5 1.5 0 0 1-1.5-1.5V5.25a1.5 1.5 0 0 1 1.5-1.5Z"
        />
      </svg>
    ),
    title: 'Actividades y entregas',
    description:
      'Publica actividades, registra entregas y consulta el estado de cada trabajo sin perder el seguimiento.',
  },
  {
    icon: (
      <svg
        aria-hidden="true"
        className="h-8 w-8"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        strokeWidth="1.8"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          d="M16.5 19.125h3.375A1.125 1.125 0 0 0 21 18v-1.125a4.5 4.5 0 0 0-4.5-4.5M16.5 19.125v-1.125a4.5 4.5 0 0 0-4.5-4.5H12a4.5 4.5 0 0 0-4.5 4.5v1.125M16.5 19.125H7.5"
        />
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          d="M15 7.875a3 3 0 1 1-6 0 3 3 0 0 1 6 0ZM16.5 12.375a3 3 0 1 0-1.875-5.343"
        />
      </svg>
    ),
    title: 'Roles y accesos',
    description:
      'Cada persona encuentra las herramientas que necesita, desde la administración hasta el aprendizaje.',
  },
]

const roles = [
  {
    title: 'Administrador',
    description:
      'Configura la plataforma y mantiene organizada la comunidad académica.',
  },
  {
    title: 'Docente',
    description:
      'Gestiona sus cursos, publica actividades y acompaña las entregas.',
  },
  {
    title: 'Estudiante',
    description:
      'Consulta sus cursos, participa en actividades y entrega sus trabajos.',
  },
]

export default function LandingPage() {
  return (
    <main className="text-verde-oscuro">
      <section className="bg-gradient-to-br from-verde-oscuro via-verde-oscuro to-verde-medio px-4 py-20 text-crema sm:px-6 sm:py-28">
        <div className="mx-auto max-w-6xl">
          <div className="max-w-3xl">
            <p className="mb-5 text-sm font-semibold uppercase tracking-[0.2em] text-dorado">
              Aprendizaje que conecta
            </p>
            <h1 className="text-4xl font-bold leading-tight sm:text-5xl lg:text-6xl">
              Aprende, enseña y crece con Classia
            </h1>
            <p className="mt-6 max-w-2xl text-lg leading-relaxed text-crema/90 sm:text-xl">
              Una plataforma virtual pensada para hacer más sencilla la
              gestión de cursos, actividades y entregas en la comunidad
              universitaria.
            </p>
            <div className="mt-9 flex flex-col gap-4 sm:flex-row">
              <Link
                to="/auth/login"
                className="rounded-lg bg-verde-medio px-7 py-3 text-center font-bold text-crema transition hover:bg-verde-medio/80 focus:outline-none focus:ring-2 focus:ring-dorado focus:ring-offset-2 focus:ring-offset-verde-oscuro"
              >
                Comenzar
              </Link>
              <a
                href="#que-es-classia"
                className="rounded-lg border border-dorado px-7 py-3 text-center font-bold text-crema transition hover:bg-dorado hover:text-verde-oscuro focus:outline-none focus:ring-2 focus:ring-dorado focus:ring-offset-2 focus:ring-offset-verde-oscuro"
              >
                Conocer más
              </a>
            </div>
          </div>
        </div>
      </section>

      <section
        id="que-es-classia"
        className="bg-crema px-4 py-16 sm:px-6 sm:py-20"
      >
        <div className="mx-auto max-w-3xl text-center">
          <p className="text-sm font-semibold uppercase tracking-[0.2em] text-dorado">
            Una experiencia académica más clara
          </p>
          <h2 className="mt-3 text-3xl font-bold sm:text-4xl">
            ¿Qué es Classia?
          </h2>
          <p className="mt-6 text-lg leading-relaxed text-verde-oscuro/80">
            Classia es un espacio digital donde docentes y estudiantes pueden
            organizar su trabajo académico de manera simple y ordenada.
            Centraliza la información de los cursos, facilita la publicación de
            actividades y permite dar seguimiento a cada entrega desde un mismo
            lugar.
          </p>
        </div>
      </section>

      <section className="bg-white px-4 py-16 sm:px-6 sm:py-20">
        <div className="mx-auto max-w-6xl">
          <div className="mx-auto max-w-2xl text-center">
            <p className="text-sm font-semibold uppercase tracking-[0.2em] text-dorado">
              Todo lo esencial
            </p>
            <h2 className="mt-3 text-3xl font-bold sm:text-4xl">
              Herramientas para avanzar juntos
            </h2>
            <p className="mt-4 text-lg text-verde-oscuro/75">
              Diseñada para que la organización no se interponga en el
              aprendizaje.
            </p>
          </div>

          <div className="mt-12 grid gap-6 md:grid-cols-3">
            {features.map((feature) => (
              <article
                key={feature.title}
                className="rounded-2xl border border-dorado/40 bg-crema/30 p-7 shadow-sm transition duration-300 hover:-translate-y-1 hover:shadow-lg"
              >
                <div className="mb-5 flex h-14 w-14 items-center justify-center rounded-xl bg-verde-medio text-dorado">
                  {feature.icon}
                </div>
                <h3 className="text-xl font-bold">{feature.title}</h3>
                <p className="mt-3 leading-relaxed text-verde-oscuro/75">
                  {feature.description}
                </p>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="bg-verde-oscuro px-4 py-16 text-crema sm:px-6 sm:py-20">
        <div className="mx-auto max-w-6xl">
          <div className="max-w-2xl">
            <p className="text-sm font-semibold uppercase tracking-[0.2em] text-dorado">
              Un espacio para cada persona
            </p>
            <h2 className="mt-3 text-3xl font-bold sm:text-4xl">
              La colaboración empieza con el rol correcto
            </h2>
          </div>

          <div className="mt-10 grid gap-8 md:grid-cols-3">
            {roles.map((role) => (
              <article
                key={role.title}
                className="border-l-2 border-dorado pl-5"
              >
                <h3 className="text-xl font-bold text-dorado">{role.title}</h3>
                <p className="mt-3 leading-relaxed text-crema/85">
                  {role.description}
                </p>
              </article>
            ))}
          </div>
        </div>
      </section>

      <section className="bg-dorado px-4 py-16 text-verde-oscuro sm:px-6 sm:py-20">
        <div className="mx-auto flex max-w-5xl flex-col items-start justify-between gap-8 md:flex-row md:items-center">
          <div className="max-w-2xl">
            <h2 className="text-3xl font-bold sm:text-4xl">
              Tu próximo paso académico empieza aquí
            </h2>
            <p className="mt-4 text-lg leading-relaxed text-verde-oscuro/80">
              Accede a Classia y descubre una forma más simple de organizar tu
              experiencia de aprendizaje.
            </p>
          </div>
          <Link
            to="/auth/login"
            className="w-full rounded-lg bg-verde-oscuro px-8 py-4 text-center text-lg font-bold text-crema transition hover:bg-verde-medio focus:outline-none focus:ring-2 focus:ring-verde-oscuro focus:ring-offset-2 focus:ring-offset-dorado md:w-auto"
          >
            Iniciar sesión
          </Link>
        </div>
      </section>
    </main>
  )
}
