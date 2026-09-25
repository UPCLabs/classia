import type { ReactNode } from 'react'
import type { ApiError } from '../../types/api'

type AsyncStateViewProps = {
  isLoading: boolean
  error: ApiError | null
  isEmpty: boolean
  children: ReactNode
}

export default function AsyncStateView({
  isLoading,
  error,
  isEmpty,
  children,
}: AsyncStateViewProps) {
  if (isLoading) {
    return (
      <div
        className="flex items-center justify-center gap-3 rounded-lg bg-verde-oscuro p-6 text-crema"
        role="status"
      >
        <span
          aria-hidden="true"
          className="h-5 w-5 animate-spin rounded-full border-2 border-crema border-t-dorado"
        />
        <span>Cargando...</span>
      </div>
    )
  }

  if (error) {
    return (
      <div
        className="rounded-lg border border-dorado bg-verde-oscuro p-6 text-crema"
        role="alert"
      >
        <p>{error.message}</p>
      </div>
    )
  }

  if (isEmpty) {
    return (
      <div className="rounded-lg bg-verde-medio p-6 text-center text-crema">
        No hay resultados.
      </div>
    )
  }

  return children
}
