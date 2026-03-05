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
import { useSaveProfile } from '../hooks/useProfiles'

interface SaveProfileDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  gameId: number
  existingNames: string[]
}

export default function SaveProfileDialog({
  open,
  onOpenChange,
  gameId,
  existingNames,
}: SaveProfileDialogProps) {
  const [name, setName] = useState('')
  const [confirmOverwrite, setConfirmOverwrite] = useState(false)
  const saveMutation = useSaveProfile()

  useEffect(() => {
    if (open) {
      setName('')
      setConfirmOverwrite(false)
    }
  }, [open])

  function handleSave() {
    const trimmed = name.trim()
    if (!trimmed) return

    // Check for duplicate name — require overwrite confirmation
    if (existingNames.includes(trimmed) && !confirmOverwrite) {
      setConfirmOverwrite(true)
      return
    }

    saveMutation.mutate(
      { gameId, name: trimmed },
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
          <DialogTitle>Save Profile</DialogTitle>
          <DialogDescription>
            Save the current mod configuration as a named profile.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3">
          <div>
            <label className="text-xs font-medium text-muted-foreground mb-1 block">
              Profile Name
            </label>
            <Input
              value={name}
              onChange={(e) => {
                setName(e.target.value)
                setConfirmOverwrite(false)
              }}
              placeholder="My Profile"
              autoFocus
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleSave()
              }}
            />
          </div>

          {confirmOverwrite && (
            <p className="text-sm text-yellow-500">
              Profile "{name.trim()}" already exists. Click Save again to overwrite.
            </p>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleSave}
            disabled={saveMutation.isPending || !name.trim()}
          >
            {saveMutation.isPending && <Loader2 className="h-4 w-4 animate-spin" />}
            {confirmOverwrite ? 'Overwrite' : 'Save'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
