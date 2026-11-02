import { useState } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { publishActivity } from '../../api/activities'
import getErrorMessage from '../../auth/getErrorMessage'
import type { Activity } from '../../types/api'

type PublishActivityButtonProps = {
  activity: Activity
}

export default function PublishActivityButton({
  activity,
}: PublishActivityButtonProps) {
  const queryClient = useQueryClient()
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const publishMutation = useMutation({
    mutationFn: () => publishActivity(activity.id),
    onSuccess: async (publishedActivity) => {
      queryClient.setQueryData<Activity[]>(
        ['activities', activity.course_id],
        (activities) =>
          activities?.map((item) =>
            item.id === publishedActivity.id ? publishedActivity : item,
          ),
      )
      queryClient.setQueryData(['activity', activity.id], publishedActivity)
      await Promise.all([
        queryClient.invalidateQueries({
          queryKey: ['activities', activity.course_id],
        }),
        queryClient.invalidateQueries({
          queryKey: ['activity', activity.id],
        }),
      ])
    },
  })

  async function handlePublish() {
    if (!window.confirm(`¿Publicar "${activity.title}" para los estudiantes?`)) {
      return
    }

    setErrorMessage(null)
    try {
      await publishMutation.mutateAsync()
    } catch (error: unknown) {
      setErrorMessage(getErrorMessage(error))
    }
  }

  if (activity.status !== 'draft' && !publishMutation.isSuccess) return null

  return (
    <div className="mt-4 flex flex-wrap items-center gap-3">
      {activity.status === 'draft' && (
        <button
          className="rounded-lg bg-verde-medio px-4 py-2 font-semibold text-crema transition hover:bg-verde-oscuro disabled:opacity-60"
          disabled={publishMutation.isPending}
          onClick={handlePublish}
          type="button"
        >
          {publishMutation.isPending ? 'Publicando...' : 'Publicar'}
        </button>
      )}
      {errorMessage && (
        <p className="text-sm text-verde-oscuro" role="alert">
          {errorMessage}
        </p>
      )}
      {publishMutation.isSuccess && (
        <p className="text-sm font-semibold text-verde-medio" role="status">
          Actividad publicada.
        </p>
      )}
    </div>
  )
}
