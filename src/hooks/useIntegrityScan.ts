import { useQuery } from '@tanstack/react-query'
import { commands } from '../lib/tauri'

/**
 * Unwrap a tauri-specta Result<T, AppError>.
 * On { status: "ok" } returns data; on { status: "error" } throws for React Query.
 */
function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: { kind: string; message: string } }): T {
  if (result.status === "ok") return result.data
  throw new Error(result.error.message)
}

export function useIntegrityScan() {
  return useQuery({
    queryKey: ['integrity-scan'],
    queryFn: async () => {
      const result = await commands.runIntegrityScan()
      return unwrap(result)
    },
    // Run once on startup — don't auto-refetch. User can manually trigger in future.
    staleTime: Infinity,
    retry: 1,
  })
}
