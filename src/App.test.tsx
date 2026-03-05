import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import React from 'react'
import App from './App'

// Mock bindings so useGames doesn't hit Tauri
vi.mock('./bindings', () => ({
  commands: {
    listGames: vi.fn().mockResolvedValue({ status: 'ok', data: [] }),
    addGame: vi.fn(),
    removeGame: vi.fn(),
    editGame: vi.fn(),
    runIntegrityScan: vi.fn(),
  },
}))

function makeWrapper() {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return ({ children }: { children: React.ReactNode }) =>
    React.createElement(QueryClientProvider, { client: qc }, children)
}

describe('App', () => {
  it('renders ModToggler text', () => {
    const Wrapper = makeWrapper()
    render(<Wrapper><App /></Wrapper>)
    expect(screen.getByText('ModToggler')).toBeDefined()
  })

  it('renders Settings button', () => {
    const Wrapper = makeWrapper()
    render(<Wrapper><App /></Wrapper>)
    expect(screen.getByLabelText('Settings')).toBeDefined()
  })
})
