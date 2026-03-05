import { useState } from 'react'
import { Trash2 } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { useProfiles, useDeleteProfile } from '../hooks/useProfiles'

interface ManageProfilesDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  gameId: number
}

export default function ManageProfilesDialog({
  open,
  onOpenChange,
  gameId,
}: ManageProfilesDialogProps) {
  const { data: profiles = [] } = useProfiles(gameId)
  const deleteMutation = useDeleteProfile()
  const [confirmDeleteId, setConfirmDeleteId] = useState<number | null>(null)

  function handleDelete(profileId: number) {
    if (confirmDeleteId === profileId) {
      deleteMutation.mutate(profileId)
      setConfirmDeleteId(null)
    } else {
      setConfirmDeleteId(profileId)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Manage Profiles</DialogTitle>
          <DialogDescription>
            View and delete saved profiles for this game.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-1">
          {profiles.length === 0 ? (
            <p className="text-sm text-muted-foreground py-4 text-center">
              No saved profiles
            </p>
          ) : (
            profiles.map((profile) => (
              <div
                key={profile.id}
                className="flex items-center justify-between px-2 py-1.5 rounded-md hover:bg-muted/50"
              >
                <span className="text-sm">{profile.name}</span>
                <Button
                  variant="ghost"
                  size="sm"
                  className={
                    confirmDeleteId === profile.id
                      ? 'text-destructive hover:text-destructive'
                      : 'text-muted-foreground hover:text-destructive'
                  }
                  onClick={() => handleDelete(profile.id)}
                  disabled={deleteMutation.isPending}
                >
                  {confirmDeleteId === profile.id ? (
                    <span className="text-xs">Delete?</span>
                  ) : (
                    <Trash2 className="h-4 w-4" />
                  )}
                </Button>
              </div>
            ))
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}
