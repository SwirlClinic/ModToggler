import { useState } from 'react'
import {
  AlertTriangle,
  ChevronDown,
  ChevronRight,
  FileText,
  FolderPlus,
  Loader2,
  Trash2,
  X,
} from 'lucide-react'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import SubModOptions from './SubModOptions'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import FileMapTable, { type FileMapping } from './FileMapTable'
import {
  useModFiles,
  useCheckConflicts,
  useToggleMod,
  useDeleteMod,
  useAddFilesToMod,
  useRemoveFileFromMod,
} from '../hooks/useMods'
import type { ModRecord, ConflictInfo } from '../bindings'

interface ModCardProps {
  mod: ModRecord
  gameId: number
  expanded: boolean
  onToggleExpand: () => void
  onConflictDetected: (conflicts: ConflictInfo[]) => void
}

export default function ModCard({
  mod,
  gameId,
  expanded,
  onToggleExpand,
  onConflictDetected,
}: ModCardProps) {
  const [deleteConfirm, setDeleteConfirm] = useState(false)
  const [addFilesOpen, setAddFilesOpen] = useState(false)
  const [addFilesMappings, setAddFilesMappings] = useState<FileMapping[]>([])

  const isLoose = mod.mod_type === 'loose'

  const { data: files = [] } = useModFiles(expanded ? mod.id : null)
  const { data: conflicts = [] } = useCheckConflicts(mod.id, gameId)
  const toggleMod = useToggleMod()
  const deleteMod = useDeleteMod()
  const addFiles = useAddFilesToMod()
  const removeFile = useRemoveFileFromMod()

  const hasConflicts = conflicts.length > 0
  const isMutating = toggleMod.isPending || deleteMod.isPending

  function handleToggle(e: React.MouseEvent) {
    e.stopPropagation()
    if (isMutating) return

    // If enabling and conflicts exist, show conflict dialog instead
    if (!mod.enabled && hasConflicts) {
      onConflictDetected(conflicts)
      return
    }

    toggleMod.mutate({ modId: mod.id, enable: !mod.enabled })
  }

  function handleDelete(e: React.MouseEvent) {
    e.stopPropagation()
    if (deleteConfirm) {
      deleteMod.mutate(mod.id)
      setDeleteConfirm(false)
    } else {
      setDeleteConfirm(true)
    }
  }

  async function handleAddFiles(e: React.MouseEvent) {
    e.stopPropagation()
    const selected = await openDialog({ multiple: true, title: 'Select files to add' })
    if (selected && Array.isArray(selected) && selected.length > 0) {
      setAddFilesMappings(
        selected.map((path) => {
          const fileName = path.split(/[/\\]/).pop() ?? path
          return { fileName, sourcePath: path, destinationPath: '/', selected: true }
        }),
      )
      setAddFilesOpen(true)
    }
  }

  function handleConfirmAddFiles() {
    if (addFilesMappings.length === 0) return
    addFiles.mutate(
      {
        modId: mod.id,
        files: addFilesMappings.map((f) => ({
          source_path: f.sourcePath,
          destination_path: f.destinationPath,
          file_name: f.fileName,
        })),
      },
      { onSuccess: () => setAddFilesOpen(false) },
    )
  }

  function handleRemoveFile(e: React.MouseEvent, fileEntryId: number) {
    e.stopPropagation()
    removeFile.mutate(fileEntryId)
  }

  function handleConflictBadgeClick(e: React.MouseEvent) {
    e.stopPropagation()
    onConflictDetected(conflicts)
  }

  return (
    <Card
      className={`cursor-pointer transition-colors hover:bg-accent/30 ${
        !mod.enabled ? 'opacity-70' : ''
      } ${expanded ? 'ring-1 ring-ring/30' : ''}`}
      onClick={onToggleExpand}
    >
      {/* Card header row */}
      <div className="flex items-center gap-3 px-4 py-3">
        {/* Expand indicator */}
        {expanded ? (
          <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground" />
        ) : (
          <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
        )}

        {/* Mod name */}
        <span className="font-medium text-sm truncate flex-1">{mod.name}</span>

        {/* Conflict badge */}
        {hasConflicts && (
          <button
            type="button"
            onClick={handleConflictBadgeClick}
            className="flex items-center gap-1 rounded-full bg-amber-500/20 text-amber-400 px-2 py-0.5 text-xs font-medium hover:bg-amber-500/30 transition-colors"
            aria-label="View conflicts"
          >
            <AlertTriangle className="h-3 w-3" />
            <span>{conflicts.length}</span>
          </button>
        )}

        {/* Status pill */}
        <span
          className={`text-xs font-medium px-2 py-0.5 rounded-full ${
            mod.enabled
              ? 'bg-emerald-500/20 text-emerald-400'
              : 'bg-muted text-muted-foreground'
          }`}
        >
          {mod.enabled ? 'Enabled' : 'Disabled'}
        </span>

        {/* Toggle switch */}
        <button
          type="button"
          disabled={isMutating}
          onClick={handleToggle}
          className={`relative inline-flex h-6 w-11 shrink-0 items-center rounded-full transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring ${
            isMutating
              ? 'opacity-50 cursor-not-allowed'
              : ''
          } ${
            mod.enabled ? 'bg-emerald-600' : 'bg-muted-foreground/30'
          }`}
          aria-label={`Toggle ${mod.name}`}
        >
          {toggleMod.isPending ? (
            <Loader2 className="h-4 w-4 animate-spin mx-auto text-foreground" />
          ) : (
            <span
              className={`inline-block h-4 w-4 rounded-full bg-white transition-transform ${
                mod.enabled ? 'translate-x-6' : 'translate-x-1'
              }`}
            />
          )}
        </button>
      </div>

      {/* Expanded content */}
      {expanded && (
        <CardContent className="pt-0 pb-4 border-t border-border">
          {/* File manifest */}
          <div className="mt-3">
            <div className="flex items-center gap-1.5 text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
              <FileText className="h-3 w-3" />
              <span>Files ({files.length})</span>
              {isLoose && (
                <button
                  type="button"
                  onClick={handleAddFiles}
                  className="ml-auto p-0.5 rounded hover:bg-muted transition-colors"
                  aria-label="Add files to mod"
                >
                  <FolderPlus className="h-3.5 w-3.5 text-muted-foreground hover:text-foreground" />
                </button>
              )}
            </div>
            {files.length > 0 ? (
              <div className="max-h-40 overflow-y-auto rounded-md bg-muted/20 p-2">
                {files.map((f) => (
                  <div
                    key={f.id}
                    className="group flex items-center gap-1 py-0.5"
                  >
                    <span
                      className="flex-1 min-w-0 text-xs text-muted-foreground font-mono truncate"
                      title={isLoose && f.destination_path ? `${f.destination_path}/${f.relative_path}` : f.relative_path}
                    >
                      {f.relative_path}
                    </span>
                    {isLoose && f.destination_path && (
                      <span
                        className="shrink-0 text-[10px] text-muted-foreground/60 font-mono bg-muted/40 px-1 rounded"
                        title={`Destination: ${f.destination_path}`}
                      >
                        {f.destination_path}
                      </span>
                    )}
                    {isLoose && (
                      <button
                        type="button"
                        onClick={(e) => handleRemoveFile(e, f.id)}
                        disabled={removeFile.isPending}
                        className="shrink-0 opacity-0 group-hover:opacity-100 p-0.5 rounded hover:bg-destructive/20 transition-all"
                        aria-label={`Remove ${f.relative_path}`}
                      >
                        <X className="h-3 w-3 text-destructive" />
                      </button>
                    )}
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-xs text-muted-foreground italic">No files</p>
            )}
          </div>

          {/* Sub-mod options (hidden for loose mods) */}
          {!isLoose && <SubModOptions modId={mod.id} parentEnabled={mod.enabled} />}

          {/* Add files dialog for loose mods */}
          {isLoose && (
            <Dialog open={addFilesOpen} onOpenChange={setAddFilesOpen}>
              <DialogContent className="max-w-md" onClick={(e) => e.stopPropagation()}>
                <DialogHeader>
                  <DialogTitle>Add Files to {mod.name}</DialogTitle>
                </DialogHeader>
                <FileMapTable files={addFilesMappings} onChange={setAddFilesMappings} />
                <DialogFooter>
                  <Button variant="outline" size="sm" onClick={() => setAddFilesOpen(false)}>
                    Cancel
                  </Button>
                  <Button
                    size="sm"
                    onClick={handleConfirmAddFiles}
                    disabled={addFiles.isPending || addFilesMappings.length === 0}
                  >
                    {addFiles.isPending && <Loader2 className="h-4 w-4 animate-spin" />}
                    Add {addFilesMappings.length} Files
                  </Button>
                </DialogFooter>
              </DialogContent>
            </Dialog>
          )}

          {/* Delete button */}
          <div className="mt-4 flex justify-end">
            <Button
              variant="destructive"
              size="sm"
              onClick={handleDelete}
              disabled={deleteMod.isPending}
            >
              {deleteMod.isPending ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Trash2 className="h-4 w-4" />
              )}
              {deleteConfirm ? 'Confirm Delete' : 'Delete Mod'}
            </Button>
          </div>
        </CardContent>
      )}
    </Card>
  )
}
