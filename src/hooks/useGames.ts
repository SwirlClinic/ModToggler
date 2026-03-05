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

export function useGames() {
  const setGames = useGameStore((s) => s.setGames)
  return useQuery({
    queryKey: ['games'],
    queryFn: async () => {
      const result = await commands.listGames()
      const games = unwrap(result)
      setGames(games)
      return games
    },
  })
}

export function useAddGame() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (args: { name: string; modDir: string; stagingDir: string | null; modStructure: string }) => {
      const result = await commands.addGame(args.name, args.modDir, args.stagingDir, args.modStructure)
      return unwrap(result)
    },
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['games'] })
      if (data.cross_drive_warning) {
        toast.warning(
          'Staging folder is on a different drive. File moves will be slower. Consider placing staging near your game directory.'
        )
      }
      if (data.has_existing_mods) {
        toast.info(
          'Existing mod files (.pak/.ucas/.utoc) detected in this directory. Open the mod list and use Import to bring them in.'
        )
      }
    },
    onError: (err: Error) => {
      toast.error(err.message ?? 'Failed to add game')
    },
  })
}

export function useRemoveGame() {
  const queryClient = useQueryClient()
  const setActiveGame = useGameStore((s) => s.setActiveGame)
  const activeGameId = useGameStore((s) => s.activeGameId)
  return useMutation({
    mutationFn: async (id: number) => {
      const result = await commands.removeGame(id)
      return unwrap(result)
    },
    onSuccess: (_data, id) => {
      if (activeGameId === id) setActiveGame(null)
      queryClient.invalidateQueries({ queryKey: ['games'] })
    },
    onError: (err: Error) => {
      toast.error(err.message ?? 'Failed to remove game')
    },
  })
}

export function useEditGame() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (args: { id: number; name: string; modDir: string; stagingDir: string; modStructure: string }) => {
      const result = await commands.editGame(args.id, args.name, args.modDir, args.stagingDir, args.modStructure)
      return unwrap(result)
    },
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['games'] })
      if (data.cross_drive_warning) {
        toast.warning('Staging folder is on a different drive after path change.')
      }
    },
    onError: (err: Error) => {
      toast.error(err.message ?? 'Failed to update game')
    },
  })
}
