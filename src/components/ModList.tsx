import { useState } from 'react'
import { Upload, Loader2 } from 'lucide-react'
import { open } from '@tauri-apps/plugin-dialog'
import { Button } from '@/components/ui/button'
import EmptyModView from './EmptyModView'
import ModCard from './ModCard'
import ImportDialog from './ImportDialog'
import ConflictDialog from './ConflictDialog'
import { useMods, useToggleMod } from '../hooks/useMods'
import { useGames } from '../hooks/useGames'
import { useGameStore } from '../store/gameStore'
import type { ConflictInfo } from '../bindings'

export default function ModList() {
  const activeGameId = useGameStore((s) => s.activeGameId)
  const { data: games = [] } = useGames()
  const { data: mods = [], isLoading } = useMods(activeGameId)
  const toggleMod = useToggleMod()

  const [expandedModId, setExpandedModId] = useState<number | null>(null)

  // Import dialog state
  const [importOpen, setImportOpen] = useState(false)
  const [importZipPath, setImportZipPath] = useState('')

  // Conflict dialog state
  const [conflictOpen, setConflictOpen] = useState(false)
  const [conflictData, setConflictData] = useState<{
    conflicts: ConflictInfo[]
    modName: string
    modId: number
    readOnly: boolean
  } | null>(null)

  const game = games.find((g) => g.id === activeGameId)

  async function handleImportClick() {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Zip Archives', extensions: ['zip'] }],
    })
    if (selected) {
      setImportZipPath(selected as string)
      setImportOpen(true)
    }
  }

  function handleConflictDetected(
    modId: number,
    modName: string,
    conflicts: ConflictInfo[],
    readOnly = false,
  ) {
    setConflictData({ conflicts, modName, modId, readOnly })
    setConflictOpen(true)
  }

  function handleEnableAnyway() {
    if (!conflictData) return
    toggleMod.mutate({ modId: conflictData.modId, enable: true })
    setConflictOpen(false)
    setConflictData(null)
  }

  function handleDisableOther(conflictingModId: number) {
    toggleMod.mutate({ modId: conflictingModId, enable: false })
    setConflictOpen(false)
    setConflictData(null)
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (mods.length === 0) {
    return <EmptyModView />
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header bar */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-border">
        <div className="text-sm text-muted-foreground">
          {game?.name} &mdash; {mods.length} mod{mods.length !== 1 ? 's' : ''}
        </div>
        <Button size="sm" onClick={handleImportClick}>
          <Upload className="h-4 w-4" />
          Import
        </Button>
      </div>

      {/* Mod cards */}
      <div className="flex-1 overflow-y-auto p-4 space-y-2">
        {mods.map((mod) => (
          <ModCard
            key={mod.id}
            mod={mod}
            gameId={activeGameId!}
            expanded={expandedModId === mod.id}
            onToggleExpand={() =>
              setExpandedModId(expandedModId === mod.id ? null : mod.id)
            }
            onConflictDetected={(conflicts) => {
              // Determine if read-only: if the mod is already enabled, this is just viewing conflicts
              const readOnly = mod.enabled
              handleConflictDetected(mod.id, mod.name, conflicts, readOnly)
            }}
          />
        ))}
      </div>

      {/* Import dialog */}
      {activeGameId && (
        <ImportDialog
          open={importOpen}
          onOpenChange={setImportOpen}
          zipPath={importZipPath}
          gameId={activeGameId}
        />
      )}

      {/* Conflict dialog */}
      {conflictData && (
        <ConflictDialog
          open={conflictOpen}
          onOpenChange={(open) => {
            setConflictOpen(open)
            if (!open) setConflictData(null)
          }}
          conflicts={conflictData.conflicts}
          modName={conflictData.modName}
          modId={conflictData.modId}
          readOnly={conflictData.readOnly}
          onEnableAnyway={handleEnableAnyway}
          onDisableOther={handleDisableOther}
        />
      )}
    </div>
  )
}
