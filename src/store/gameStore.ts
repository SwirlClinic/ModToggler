import { create } from 'zustand'
import type { GameRecord } from '../bindings'

interface GameStore {
  games: GameRecord[]
  activeGameId: number | null
  setGames: (games: GameRecord[]) => void
  setActiveGame: (id: number | null) => void
}

export const useGameStore = create<GameStore>((set) => ({
  games: [],
  activeGameId: null,
  setGames: (games) => set({ games }),
  setActiveGame: (id) => set({ activeGameId: id }),
}))
