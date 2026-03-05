import { useState, useEffect } from 'react'
import { Loader2 } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { useImportMod } from '../hooks/useMods'

interface ImportDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  zipPath: string
  gameId: number
}

function nameFromZip(zipPath: string): string {
  const filename = zipPath.split(/[/\\]/).pop() ?? ''
  return filename.replace(/\.zip$/i, '').replace(/[-_]/g, ' ')
}

export default function ImportDialog({
  open,
  onOpenChange,
  zipPath,
  gameId,
}: ImportDialogProps) {
  const [modName, setModName] = useState('')
  const importMod = useImportMod()

  useEffect(() => {
    if (open && zipPath) {
      setModName(nameFromZip(zipPath))
    }
  }, [open, zipPath])

  function handleImport() {
    if (!modName.trim()) return
    importMod.mutate(
      { gameId, zipPath, modName: modName.trim() },
      {
        onSuccess: () => {
          onOpenChange(false)
        },
      },
    )
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Import Mod</DialogTitle>
          <DialogDescription>
            Choose a name for this mod. It will be imported into your staging directory.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3">
          <div>
            <label className="text-xs font-medium text-muted-foreground mb-1 block">
              Mod Name
            </label>
            <Input
              value={modName}
              onChange={(e) => setModName(e.target.value)}
              placeholder="My Mod"
              autoFocus
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleImport()
              }}
            />
          </div>

          <div>
            <label className="text-xs font-medium text-muted-foreground mb-1 block">
              Archive
            </label>
            <p className="text-xs text-muted-foreground font-mono truncate" title={zipPath}>
              {zipPath}
            </p>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleImport}
            disabled={importMod.isPending || !modName.trim()}
          >
            {importMod.isPending && <Loader2 className="h-4 w-4 animate-spin" />}
            Import
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
