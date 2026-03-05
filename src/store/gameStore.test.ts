import { describe, it, expect, beforeEach } from 'vitest'
import { useGameStore } from './gameStore'
import type { GameRecord } from '../bindings'

const game: GameRecord = {
  id: 1,
  name: 'Tekken 8',
  mod_dir: 'C:/Mods',
  staging_dir: 'C:/staging',
  mod_structure: 'structured',
  requires_elevation: false,
}

describe('gameStore', () => {
  beforeEach(() => useGameStore.setState({ games: [], activeGameId: null }))

  it('setActiveGame updates activeGameId', () => {
    useGameStore.getState().setActiveGame(1)
    expect(useGameStore.getState().activeGameId).toBe(1)
  })

  it('setActiveGame(null) resets activeGameId', () => {
    useGameStore.getState().setActiveGame(1)
    useGameStore.getState().setActiveGame(null)
    expect(useGameStore.getState().activeGameId).toBeNull()
  })

  it('setGames replaces the games array', () => {
    useGameStore.getState().setGames([game])
    expect(useGameStore.getState().games).toHaveLength(1)
    expect(useGameStore.getState().games[0].name).toBe('Tekken 8')
  })
})
