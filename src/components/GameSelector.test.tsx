import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import GameSelector from './GameSelector'
import { useGameStore } from '../store/gameStore'
import type { GameRecord } from '../bindings'

const mockGames: GameRecord[] = [
  { id: 1, name: 'Tekken 8', mod_dir: 'C:/Mods', staging_dir: 'C:/staging', mod_structure: 'structured', requires_elevation: false },
  { id: 2, name: 'Dark Souls', mod_dir: 'D:/Mods', staging_dir: 'D:/staging', mod_structure: 'structured', requires_elevation: false },
]

describe('GameSelector', () => {
  beforeEach(() => useGameStore.setState({ games: [], activeGameId: null }))

  it('shows empty state when no games', () => {
    render(<GameSelector games={[]} />)
    expect(screen.getByText(/No games/i)).toBeInTheDocument()
  })

  it('renders game names in the list', () => {
    render(<GameSelector games={mockGames} />)
    expect(screen.getByRole('combobox')).toBeInTheDocument()
  })
})
