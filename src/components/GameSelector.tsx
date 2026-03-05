import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { useGameStore } from '../store/gameStore'
import type { GameRecord } from '../bindings'

interface Props {
  games: GameRecord[]
}

export default function GameSelector({ games }: Props) {
  const { activeGameId, setActiveGame } = useGameStore()

  if (games.length === 0) {
    return (
      <span className="text-muted-foreground text-xs">
        No games — add one in Settings
      </span>
    )
  }

  return (
    <Select
      value={activeGameId != null ? String(activeGameId) : ''}
      onValueChange={(val) => setActiveGame(val ? Number(val) : null)}
    >
      <SelectTrigger className="w-48 h-8 text-sm">
        <SelectValue placeholder="Select a game" />
      </SelectTrigger>
      <SelectContent>
        {games.map((g) => (
          <SelectItem key={g.id} value={String(g.id)}>
            <span className="flex items-center gap-1.5">
              {g.name}
              {g.mod_structure === 'loose' && (
                <span className="text-[10px] font-medium bg-blue-500/20 text-blue-400 px-1.5 py-0.5 rounded">
                  Loose
                </span>
              )}
            </span>
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}
