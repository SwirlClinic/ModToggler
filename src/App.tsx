import { useState } from 'react'
import { Settings } from 'lucide-react'
import { Button } from '@/components/ui/button'
import GameSelector from './components/GameSelector'
import SettingsPanel from './components/SettingsPanel'
import ModList from './components/ModList'
import IntegrityAlert from './components/IntegrityAlert'
import { useGameStore } from './store/gameStore'
import { useGames } from './hooks/useGames'

export default function App() {
  const [settingsOpen, setSettingsOpen] = useState(false)
  const { activeGameId } = useGameStore()
  const { data: games = [] } = useGames()

  return (
    <div className="min-h-screen bg-background text-foreground flex flex-col" data-theme="dark">
      {/* Top bar */}
      <header className="h-12 border-b border-border flex items-center px-4 gap-3 shrink-0">
        <span className="font-semibold text-sm tracking-wide">ModToggler</span>
        <div className="flex-1">
          <GameSelector games={games} />
        </div>
        <Button
          variant="ghost"
          size="icon"
          aria-label="Settings"
          onClick={() => setSettingsOpen(true)}
        >
          <Settings className="h-4 w-4" />
        </Button>
      </header>

      {/* Main content */}
      <main className="flex-1 overflow-auto">
        <IntegrityAlert />
        {activeGameId == null ? (
          <div className="flex items-center justify-center h-full text-muted-foreground text-sm">
            Select a game to get started
          </div>
        ) : (
          <ModList />
        )}
      </main>

      <SettingsPanel open={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </div>
  )
}
