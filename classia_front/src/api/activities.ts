import httpClient from './httpClient'
import type {
  Activity,
  ActivityCreate,
  ActivityUpdate,
  ListResponse,
} from '../types/api'

const BOGOTA_OFFSET_MS = 5 * 60 * 60 * 1000

export async function createActivity(
  courseId: string,
  activity: ActivityCreate,
): Promise<Activity> {
  const { data } = await httpClient.post<Activity>(
    `/courses/${courseId}/activities`,
    activity,
  )
  return data
}

export async function getActivity(activityId: string): Promise<Activity> {
  const { data } = await httpClient.get<Activity>(`/activities/${activityId}`)
  return data
}

export async function listCourseActivities(courseId: string): Promise<Activity[]> {
  const { data } = await httpClient.get<ListResponse<Activity>>(
    `/courses/${courseId}/activities`,
  )
  return data.items
}

export async function updateActivity(
  activityId: string,
  changes: ActivityUpdate,
): Promise<Activity> {
  const { data } = await httpClient.patch<Activity>(
    `/activities/${activityId}`,
    changes,
  )
  return data
}

export async function publishActivity(activityId: string): Promise<Activity> {
  const { data } = await httpClient.post<Activity>(
    `/activities/${activityId}/publish`,
  )
  return data
}

export function bogotaDateTimeToUtc(value: string): string {
  const [date, time] = value.split('T')
  const [year, month, day] = date.split('-').map(Number)
  const [hours, minutes] = time.split(':').map(Number)

  return new Date(
    Date.UTC(year, month - 1, day, hours, minutes) + BOGOTA_OFFSET_MS,
  ).toISOString()
}

export function utcToBogotaDateTime(value: string): string {
  const date = new Date(new Date(value).getTime() - BOGOTA_OFFSET_MS)
  const pad = (part: number) => String(part).padStart(2, '0')

  return `${date.getUTCFullYear()}-${pad(date.getUTCMonth() + 1)}-${pad(date.getUTCDate())}T${pad(date.getUTCHours())}:${pad(date.getUTCMinutes())}`
}

/** `value` llega como fecha y hora de Bogotá en formato `datetime-local`. */
export function isBogotaDateTimeInFuture(
  value: string,
  now: number = Date.now(),
): boolean {
  return new Date(bogotaDateTimeToUtc(value)).getTime() > now
}
