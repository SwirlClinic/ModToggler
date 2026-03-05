import { useState } from 'react'
import { Plus, Pencil, Trash2 } from 'lucide-react'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import GameForm from './GameForm'
import { useGames, useAddGame, useRemoveGame, useEditGame } from '../hooks/useGames'
import type { GameRecord } from '../bindings'

interface Props {
  open: boolean
  onClose: () => void
}

export default function SettingsPanel({ open, onClose }: Props) {
  const [addOpen, setAddOpen] = useState(false)
  const [editTarget, setEditTarget] = useState<GameRecord | null>(null)
  const { data: games = [] } = useGames()
  const addGame = useAddGame()
  const removeGame = useRemoveGame()
  const editGame = useEditGame()

  return (
    <>
      <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
        <DialogContent className="sm:max-w-lg">
          <DialogHeader>
            <DialogTitle>Settings</DialogTitle>
          </DialogHeader>

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-medium">Games</h3>
              <Button size="sm" variant="outline" onClick={() => setAddOpen(true)}>
                <Plus className="h-3 w-3 mr-1" /> Add Game
              </Button>
            </div>

            <Separator />

            {games.length === 0 ? (
              <p className="text-sm text-muted-foreground py-4 text-center">
                No games added yet.
              </p>
            ) : (
              <ul className="space-y-2">
                {games.map((g) => (
                  <li key={g.id} className="flex items-center justify-between p-2 rounded border border-border">
                    <div>
                      <p className="text-sm font-medium">{g.name}</p>
                      <p className="text-xs text-muted-foreground truncate max-w-xs">{g.mod_dir}</p>
                    </div>
                    <div className="flex gap-1 shrink-0">
                      <Button variant="ghost" size="icon" onClick={() => setEditTarget(g)}>
                        <Pencil className="h-3.5 w-3.5" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="text-destructive hover:text-destructive"
                        onClick={() => removeGame.mutate(g.id)}
                        disabled={removeGame.isPending}
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </Button>
                    </div>
                  </li>
                ))}
              </ul>
            )}
          </div>
        </DialogContent>
      </Dialog>

      <GameForm
        open={addOpen}
        onClose={() => setAddOpen(false)}
        onSubmit={async (data) => {
          await addGame.mutateAsync(data)
          setAddOpen(false)
        }}
        loading={addGame.isPending}
      />

      {editTarget && (
        <GameForm
          open={true}
          onClose={() => setEditTarget(null)}
          existing={editTarget}
          onSubmit={async (data) => {
            await editGame.mutateAsync({
              id: editTarget.id,
              name: data.name,
              modDir: data.modDir,
              stagingDir: data.stagingDir ?? editTarget.staging_dir,
              modStructure: data.modStructure,
            })
            setEditTarget(null)
          }}
          loading={editGame.isPending}
        />
      )}
    </>
  )
}
