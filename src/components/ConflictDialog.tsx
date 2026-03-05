import { AlertTriangle } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import type { ConflictInfo } from '../bindings'

interface ConflictDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  conflicts: ConflictInfo[]
  modName: string
  modId: number
  readOnly?: boolean
  onEnableAnyway: () => void
  onDisableOther: (conflictingModId: number) => void
}

/** Group conflicts by conflicting mod */
function groupByMod(conflicts: ConflictInfo[]) {
  const map = new Map<number, { name: string; paths: string[] }>()
  for (const c of conflicts) {
    const entry = map.get(c.conflicting_mod_id)
    if (entry) {
      entry.paths.push(c.relative_path)
    } else {
      map.set(c.conflicting_mod_id, {
        name: c.conflicting_mod_name,
        paths: [c.relative_path],
      })
    }
  }
  return map
}

export default function ConflictDialog({
  open,
  onOpenChange,
  conflicts,
  modName,
  modId: _modId,
  readOnly = false,
  onEnableAnyway,
  onDisableOther,
}: ConflictDialogProps) {
  const grouped = groupByMod(conflicts)
  const uniqueConflictingMods = Array.from(grouped.entries())
  const singleConflict = uniqueConflictingMods.length === 1

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-amber-400" />
            <DialogTitle>Conflict Warning</DialogTitle>
          </div>
          <DialogDescription>
            {readOnly
              ? `"${modName}" has file conflicts with other mods.`
              : `Enabling "${modName}" will conflict with the following mod${uniqueConflictingMods.length > 1 ? 's' : ''}:`}
          </DialogDescription>
        </DialogHeader>

        <div className="max-h-60 overflow-y-auto space-y-3">
          {uniqueConflictingMods.map(([modId, { name, paths }]) => (
            <div key={modId} className="rounded-md bg-muted/30 p-3">
              <p className="text-sm font-medium text-amber-400 mb-1">{name}</p>
              <div className="space-y-0.5">
                {paths.map((p) => (
                  <p
                    key={p}
                    className="text-xs text-muted-foreground font-mono truncate"
                    title={p}
                  >
                    {p}
                  </p>
                ))}
              </div>
            </div>
          ))}
        </div>

        <DialogFooter>
          {readOnly ? (
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              Close
            </Button>
          ) : (
            <>
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                Cancel
              </Button>
              {singleConflict && (
                <Button
                  variant="secondary"
                  onClick={() =>
                    onDisableOther(uniqueConflictingMods[0][0])
                  }
                >
                  Disable Other
                </Button>
              )}
              <Button onClick={onEnableAnyway}>Enable Anyway</Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
