import { PackageOpen } from 'lucide-react'
import { useGameStore } from '../store/gameStore'
import { useGames } from '../hooks/useGames'

export default function EmptyModView() {
  const activeGameId = useGameStore((s) => s.activeGameId)
  const { data: games = [] } = useGames()
  const game = games.find((g) => g.id === activeGameId)

  return (
    <div className="flex flex-col items-center justify-center h-full gap-4 text-muted-foreground py-20">
      <PackageOpen className="h-12 w-12 opacity-30" />
      <div className="text-center">
        <p className="text-base font-medium text-foreground">No mods yet</p>
        <p className="text-sm mt-1">
          {game ? `Add mods for ${game.name} to get started` : 'Select a game to view its mods'}
        </p>
      </div>
    </div>
  )
}
