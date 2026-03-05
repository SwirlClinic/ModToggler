import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { commands } from '../lib/tauri'
import type { LooseFileInput } from '../bindings'

/**
 * Unwrap a tauri-specta Result<T, AppError>.
 * On { status: "ok" } returns data; on { status: "error" } throws for React Query.
 */
function unwrap<T>(result: { status: "ok"; data: T } | { status: "error"; error: { kind: string; message: string } }): T {
  if (result.status === "ok") return result.data
  throw new Error(result.error.message)
}

// ── Queries ──

export function useMods(gameId: number | null) {
  return useQuery({
    queryKey: ['mods', gameId],
    queryFn: async () => {
      if (!gameId) return []
      return unwrap(await commands.listMods(gameId))
    },
    enabled: gameId !== null,
  })
}

export function useModFiles(modId: number | null) {
  return useQuery({
    queryKey: ['mod-files', modId],
    queryFn: async () => {
      if (!modId) return []
      return unwrap(await commands.listModFiles(modId))
    },
    enabled: modId !== null,
  })
}

export function useSubMods(modId: number | null) {
  return useQuery({
    queryKey: ['sub-mods', modId],
    queryFn: async () => {
      if (!modId) return []
      return unwrap(await commands.listSubModsCmd(modId))
    },
    enabled: modId !== null,
  })
}

export function useCheckConflicts(modId: number | null, gameId: number | null) {
  return useQuery({
    queryKey: ['conflicts', modId, gameId],
    queryFn: async () => {
      if (!modId || !gameId) return []
      return unwrap(await commands.checkConflictsCmd(modId, gameId))
    },
    enabled: modId !== null && gameId !== null,
  })
}

// ── Mutations ──

export function useImportMod() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (args: { gameId: number; zipPath: string; modName: string }) => {
      return unwrap(await commands.importMod(args.gameId, args.zipPath, args.modName))
    },
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['mods'] })
      if (!data.has_recognized_files) {
        toast.warning('No recognized mod files (.pak/.ucas/.utoc) found in this archive')
      }
      toast.success(`Imported "${data.mod_record.name}" with ${data.file_count} files`)
    },
    onError: (err: Error) => {
      toast.error(err.message ?? 'Import failed')
    },
  })
}

export function useToggleMod() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (args: { modId: number; enable: boolean }) => {
      return unwrap(await commands.toggleModCmd(args.modId, args.enable))
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['mods'] })
      queryClient.invalidateQueries({ queryKey: ['sub-mods'] })
      queryClient.invalidateQueries({ queryKey: ['conflicts'] })
    },
    onError: (err: Error) => {
      toast.error(err.message ?? 'Toggle failed')
    },
  })
}

export function useToggleSubMod() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (args: { subModId: number; enable: boolean }) => {
      return unwrap(await commands.toggleSubModCmd(args.subModId, args.enable))
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sub-mods'] })
      queryClient.invalidateQueries({ queryKey: ['conflicts'] })
    },
    onError: (err: Error) => {
      toast.error(err.message ?? 'Sub-mod toggle failed')
    },
  })
}

export function useDeleteMod() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (modId: number) => {
      return unwrap(await commands.deleteModCmd(modId))
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['mods'] })
      queryClient.invalidateQueries({ queryKey: ['sub-mods'] })
      queryClient.invalidateQueries({ queryKey: ['conflicts'] })
      queryClient.invalidateQueries({ queryKey: ['mod-files'] })
    },
    onError: (err: Error) => {
      toast.error(err.message ?? 'Delete failed')
    },
  })
}

// ── Loose-File Mutations ──

export function useImportLooseFiles() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (args: { gameId: number; modName: string; files: LooseFileInput[] }) => {
      return unwrap(await commands.importLooseFiles(args.gameId, args.modName, args.files))
    },
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['mods'] })
      toast.success(`Imported "${data.mod_record.name}" with ${data.file_count} files`)
    },
    onError: (err: Error) => {
      toast.error(err.message ?? 'Import failed')
    },
  })
}

export function useImportLooseZip() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (args: { gameId: number; zipPath: string; modName: string; selectedFiles: LooseFileInput[] }) => {
      return unwrap(await commands.importLooseZip(args.gameId, args.zipPath, args.modName, args.selectedFiles))
    },
    onSuccess: (data) => {
      queryClient.invalidateQueries({ queryKey: ['mods'] })
      toast.success(`Imported "${data.mod_record.name}" with ${data.file_count} files`)
    },
    onError: (err: Error) => {
      toast.error(err.message ?? 'Zip import failed')
    },
  })
}

export function useAddFilesToMod() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (args: { modId: number; files: LooseFileInput[] }) => {
      return unwrap(await commands.addFilesToMod(args.modId, args.files))
    },
    onSuccess: (fileCount) => {
      queryClient.invalidateQueries({ queryKey: ['mods'] })
      queryClient.invalidateQueries({ queryKey: ['mod-files'] })
      toast.success(`Added ${fileCount} files to mod`)
    },
    onError: (err: Error) => {
      toast.error(err.message ?? 'Failed to add files')
    },
  })
}

export function useRemoveFileFromMod() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (fileEntryId: number) => {
      return unwrap(await commands.removeFileFromMod(fileEntryId))
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['mod-files'] })
      queryClient.invalidateQueries({ queryKey: ['mods'] })
      queryClient.invalidateQueries({ queryKey: ['conflicts'] })
      toast.success('File removed from mod')
    },
    onError: (err: Error) => {
      toast.error(err.message ?? 'Failed to remove file')
    },
  })
}
