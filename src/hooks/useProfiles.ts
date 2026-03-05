import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { commands } from '../lib/tauri'
import { useGameStore } from '../store/gameStore'

/**
 * Unwrap a tauri-specta Result<T, AppError>.
 * On { status: "ok" } returns data; on { status: "error" } throws for React Query.
 */
function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: { kind: string; message: string } }): T {
  if (result.status === "ok") return result.data
  throw new Error(result.error.message)
}

// ── Queries ──

export function useProfiles(gameId: number | null) {
  return useQuery({
    queryKey: ['profiles', gameId],
    queryFn: async () => {
      if (!gameId) return []
      return unwrap(await commands.listProfilesCmd(gameId))
    },
    enabled: gameId !== null,
  })
}

// ── Mutations ──

export function useSaveProfile() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (args: { gameId: number; name: string }) => {
      return unwrap(await commands.saveProfileCmd(args.gameId, args.name))
    },
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['profiles'] })
      toast.success(`Saved profile "${data.name}"`)
    },
    onError: (err: Error) => {
      toast.error(err.message ?? 'Save profile failed')
    },
  })
}

export function useLoadProfile() {
  const queryClient = useQueryClient()
  const setLastLoadedProfileName = useGameStore((s) => s.setLastLoadedProfileName)
  return useMutation({
    mutationFn: async (args: { profileId: number; profileName: string }) => {
      const result = unwrap(await commands.loadProfileCmd(args.profileId))
      return { result, profileName: args.profileName }
    },
    onSuccess: ({ result, profileName }) => {
      queryClient.invalidateQueries({ queryKey: ['mods'] })
      queryClient.invalidateQueries({ queryKey: ['sub-mods'] })
      queryClient.invalidateQueries({ queryKey: ['conflicts'] })
      setLastLoadedProfileName(profileName)
      if (result.skipped_mods.length > 0) {
        toast.warning(`Loaded "${profileName}" — skipped missing mods: ${result.skipped_mods.join(', ')}`)
      } else {
        toast.success(`Loaded profile "${profileName}"`)
      }
    },
    onError: (err: Error) => {
      toast.error(err.message ?? 'Load profile failed')
    },
  })
}

export function useDeleteProfile() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (profileId: number) => {
      return unwrap(await commands.deleteProfileCmd(profileId))
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['profiles'] })
      toast.success('Profile deleted')
    },
    onError: (err: Error) => {
      toast.error(err.message ?? 'Delete profile failed')
    },
  })
}
