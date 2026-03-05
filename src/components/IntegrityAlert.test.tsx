import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import React from 'react'
import IntegrityAlert from './IntegrityAlert'
import type { IntegrityScanResult } from '../bindings'

// Mock the tauri.ts re-export module — commands.runIntegrityScan() returns Result<T, E>
const mockRunIntegrityScan = vi.fn()
vi.mock('../lib/tauri', () => ({
  commands: {
    runIntegrityScan: () => mockRunIntegrityScan(),
  },
}))

function makeWrapper() {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  })
  return ({ children }: { children: React.ReactNode }) =>
    React.createElement(QueryClientProvider, { client: qc }, children)
}

const cleanScan: IntegrityScanResult = {
  missing_from_game: [],
  missing_from_staging: [],
  incomplete_journals: [],
}

describe('IntegrityAlert', () => {
  beforeEach(() => vi.clearAllMocks())

  it('renders nothing when scan is clean', async () => {
    mockRunIntegrityScan.mockResolvedValue({
      status: 'ok',
      data: cleanScan,
    })
    const { container } = render(<IntegrityAlert />, {
      wrapper: makeWrapper(),
    })
    // Wait for query to settle — should remain empty
    await waitFor(() => {
      expect(mockRunIntegrityScan).toHaveBeenCalled()
    })
    // Give React time to re-render after data arrives
    await new Promise((r) => setTimeout(r, 50))
    expect(container).toBeEmptyDOMElement()
  })

  it('renders alert when incomplete journals exist', async () => {
    mockRunIntegrityScan.mockResolvedValue({
      status: 'ok',
      data: {
        ...cleanScan,
        incomplete_journals: [
          { id: 1, mod_id: 1, operation: 'enable', files: [] },
        ],
      },
    })
    const { findByRole } = render(<IntegrityAlert />, {
      wrapper: makeWrapper(),
    })
    const alert = await findByRole('alert')
    expect(alert).toBeInTheDocument()
    expect(alert.textContent).toContain('incomplete toggle operation')
  })

  it('renders alert listing mod name when missing from game', async () => {
    mockRunIntegrityScan.mockResolvedValue({
      status: 'ok',
      data: {
        ...cleanScan,
        missing_from_game: [
          {
            id: 1,
            game_id: 1,
            name: 'TestMod',
            enabled: true,
            staged_path: '/staging',
          },
        ],
      },
    })
    const { findByRole } = render(<IntegrityAlert />, {
      wrapper: makeWrapper(),
    })
    const alert = await findByRole('alert')
    expect(alert.textContent).toContain('TestMod')
  })

  it('dismiss button hides the alert', async () => {
    mockRunIntegrityScan.mockResolvedValue({
      status: 'ok',
      data: {
        ...cleanScan,
        incomplete_journals: [
          { id: 1, mod_id: 1, operation: 'enable', files: [] },
        ],
      },
    })
    const { findByRole, queryByRole } = render(<IntegrityAlert />, {
      wrapper: makeWrapper(),
    })
    await findByRole('alert')
    const dismissBtn = screen.getByLabelText('Dismiss integrity alert')
    fireEvent.click(dismissBtn)
    expect(queryByRole('alert')).toBeNull()
  })
})
