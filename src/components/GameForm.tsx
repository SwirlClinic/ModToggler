import { useState } from 'react'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { FolderOpen } from 'lucide-react'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { GameRecord } from '../bindings'

interface Props {
  open: boolean
  onClose: () => void
  onSubmit: (data: { name: string; modDir: string; stagingDir: string | null; modStructure: string }) => void
  existing?: GameRecord
  loading?: boolean
}

export default function GameForm({ open, onClose, onSubmit, existing, loading }: Props) {
  const [name, setName] = useState(existing?.name ?? '')
  const [modDir, setModDir] = useState(existing?.mod_dir ?? '')
  const [stagingDir, setStagingDir] = useState(existing?.staging_dir ?? '')
  const [modStructure, setModStructure] = useState<string>(existing?.mod_structure ?? 'structured')

  const handlePickModDir = async () => {
    const selected = await openDialog({ directory: true, multiple: false, title: 'Select Mod Directory' })
    if (selected) setModDir(selected as string)
  }

  const handlePickStagingDir = async () => {
    const selected = await openDialog({ directory: true, multiple: false, title: 'Select Staging Directory' })
    if (selected) setStagingDir(selected as string)
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!name.trim() || !modDir.trim()) return
    onSubmit({
      name: name.trim(),
      modDir: modDir.trim(),
      stagingDir: stagingDir.trim() || null,
      modStructure,
    })
  }

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{existing ? 'Edit Game' : 'Add Game'}</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div className="space-y-1">
            <Label htmlFor="name">Game Name</Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. Tekken 8"
              required
            />
          </div>

          <div className="space-y-1">
            <Label htmlFor="mod-dir">Mod Directory</Label>
            <div className="flex gap-2">
              <Input
                id="mod-dir"
                value={modDir}
                onChange={(e) => setModDir(e.target.value)}
                placeholder="C:\Game\Mods"
                className="flex-1"
                required
              />
              <Button type="button" variant="outline" size="icon" onClick={handlePickModDir} title="Browse">
                <FolderOpen className="h-4 w-4" />
              </Button>
            </div>
          </div>

          <div className="space-y-1">
            <Label htmlFor="staging-dir">Staging Directory <span className="text-muted-foreground text-xs">(optional -- default used if blank)</span></Label>
            <div className="flex gap-2">
              <Input
                id="staging-dir"
                value={stagingDir}
                onChange={(e) => setStagingDir(e.target.value)}
                placeholder="Leave blank for default (~/.modtoggler/...)"
                className="flex-1"
              />
              <Button type="button" variant="outline" size="icon" onClick={handlePickStagingDir} title="Browse">
                <FolderOpen className="h-4 w-4" />
              </Button>
            </div>
          </div>

          <div className="space-y-1">
            <Label htmlFor="mod-structure">Mod Structure</Label>
            <Select value={modStructure} onValueChange={setModStructure}>
              <SelectTrigger id="mod-structure">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="structured">Structured (PAK/UCAS/UTOC groups)</SelectItem>
                <SelectItem value="loose">Loose Files (manual tagging)</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <DialogFooter>
            <Button type="button" variant="ghost" onClick={onClose}>Cancel</Button>
            <Button type="submit" disabled={loading}>
              {loading ? 'Saving...' : (existing ? 'Save Changes' : 'Add Game')}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
