import { useMemo } from 'react'
import type { ApiError } from '../../types/api'

export type AsyncState<T> = {
  data: T | null
  isLoading: boolean
  error: ApiError | null
  isEmpty: boolean
}

type UseAsyncStateOptions<T> = {
  data?: T | null
  isLoading: boolean
  error?: ApiError | null
}

export function useAsyncState<T>({
  data,
  isLoading,
  error = null,
}: UseAsyncStateOptions<T>): AsyncState<T> {
  return useMemo(
    () => ({
      data: data ?? null,
      isLoading,
      error,
      isEmpty: Array.isArray(data) && data.length === 0,
    }),
    [data, error, isLoading],
  )
}
