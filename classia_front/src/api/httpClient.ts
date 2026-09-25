import axios from 'axios'
import type { ApiError } from '../types/api'

const isApiError = (value: unknown): value is ApiError => {
  if (typeof value !== 'object' || value === null) {
    return false
  }

  const data = value as Record<string, unknown>
  return typeof data.code === 'string' && typeof data.message === 'string'
}

const httpClient = axios.create({
  baseURL: '/api',
  withCredentials: true,
})

httpClient.interceptors.response.use(
  (response) => response,
  (error: unknown) => {
    if (!axios.isAxiosError(error)) {
      return Promise.reject(error)
    }

    if (error.response?.status === 401) {
      window.location.href = '/login'
    }

    if (isApiError(error.response?.data)) {
      return Promise.reject(error.response.data)
    }

    return Promise.reject(error)
  },
)

export default httpClient
