import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import React from 'react'
import type { GameRecord, Result, AppError } from '../bindings'

// Mock the bindings module — commands is an object, not individual exports
vi.mock('../bindings', () => ({
  commands: {
    listGames: vi.fn(),
    addGame: vi.fn(),
    removeGame: vi.fn(),
    editGame: vi.fn(),
    runIntegrityScan: vi.fn(),
  },
}))

import { commands } from '../bindings'
import { useGames } from './useGames'

const mockGame: GameRecord = {
  id: 1,
  name: 'Tekken 8',
  mod_dir: 'C:/game/Mods',
  staging_dir: 'C:/staging',
  mod_structure: 'structured',
  requires_elevation: false,
}

function makeWrapper() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return ({ children }: { children: React.ReactNode }) =>
    React.createElement(QueryClientProvider, { client: qc }, children)
}

describe('useGames', () => {
  beforeEach(() => vi.clearAllMocks())

  it('returns game records when listGames resolves ok', async () => {
    const result: Result<GameRecord[], AppError> = { status: 'ok', data: [mockGame] }
    vi.mocked(commands.listGames).mockResolvedValue(result)
    const { result: hookResult } = renderHook(() => useGames(), { wrapper: makeWrapper() })
    await waitFor(() => expect(hookResult.current.isSuccess).toBe(true))
    expect(hookResult.current.data).toEqual([mockGame])
  })
})
