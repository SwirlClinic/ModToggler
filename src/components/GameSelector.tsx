import type { GameRecord } from '../bindings'

interface Props {
  games: GameRecord[]
}

// Stub — full implementation in Task 2
export default function GameSelector({ games }: Props) {
  return <span className="text-muted-foreground text-xs">Game selector (stub)</span>
}
