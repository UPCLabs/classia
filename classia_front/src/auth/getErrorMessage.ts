export default function getErrorMessage(error: unknown): string {
  if (typeof error === 'object' && error !== null && 'message' in error) {
    const message = error.message
    if (typeof message === 'string') {
      return message
    }
  }

  return 'Ocurrió un error inesperado. Inténtalo de nuevo.'
}
