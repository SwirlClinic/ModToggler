import { useState } from 'react'
import {
  AlertTriangle,
  ChevronDown,
  ChevronRight,
  FileText,
  Loader2,
  Trash2,
} from 'lucide-react'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import SubModOptions from './SubModOptions'
import {
  useModFiles,
  useCheckConflicts,
  useToggleMod,
  useDeleteMod,
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

  const { data: files = [] } = useModFiles(expanded ? mod.id : null)
  const { data: conflicts = [] } = useCheckConflicts(mod.id, gameId)
  const toggleMod = useToggleMod()
  const deleteMod = useDeleteMod()

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
            </div>
            {files.length > 0 ? (
              <div className="max-h-40 overflow-y-auto rounded-md bg-muted/20 p-2">
                {files.map((f) => (
                  <div
                    key={f.id}
                    className="text-xs text-muted-foreground font-mono py-0.5 truncate"
                    title={f.relative_path}
                  >
                    {f.relative_path}
                  </div>
                ))}
              </div>
            ) : (
              <p className="text-xs text-muted-foreground italic">No files</p>
            )}
          </div>

          {/* Sub-mod options */}
          <SubModOptions modId={mod.id} parentEnabled={mod.enabled} />

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
