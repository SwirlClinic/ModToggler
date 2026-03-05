import { create } from 'zustand'
import type { GameRecord } from '../bindings'

interface GameStore {
  games: GameRecord[]
  activeGameId: number | null
  lastLoadedProfileName: string | null
  setGames: (games: GameRecord[]) => void
  setActiveGame: (id: number | null) => void
  setLastLoadedProfileName: (name: string | null) => void
}

export const useGameStore = create<GameStore>((set) => ({
  games: [],
  activeGameId: null,
  lastLoadedProfileName: null,
  setGames: (games) => set({ games }),
  setActiveGame: (id) => set({ activeGameId: id, lastLoadedProfileName: null }),
  setLastLoadedProfileName: (name) => set({ lastLoadedProfileName: name }),
}))
