import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import { check } from '@tauri-apps/plugin-updater'
import type { Update } from '@tauri-apps/plugin-updater'
import { useUpdateStore } from '../store/updateStore'

vi.mock('@tauri-apps/plugin-updater', () => ({
  check: vi.fn(),
}))

const mockedCheck = vi.mocked(check)

describe('useUpdateChecker', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useUpdateStore.setState({
      status: 'idle',
      update: null,
      downloaded: 0,
      contentLength: 0,
      error: null,
    })
  })

  // Dynamically import to get fresh module after mocks are set up
  async function loadHook() {
    const mod = await import('./useUpdateChecker')
    return mod.useUpdateChecker
  }

  it('calls check() and transitions to available when update exists', async () => {
    const fakeUpdate = {
      version: '2.0.0',
      body: 'release notes',
      close: vi.fn(),
    } as unknown as Update

    mockedCheck.mockResolvedValue(fakeUpdate)

    const useUpdateChecker = await loadHook()
    renderHook(() => useUpdateChecker())

    await waitFor(() => {
      expect(useUpdateStore.getState().status).toBe('available')
    })
    expect(mockedCheck).toHaveBeenCalledOnce()
    expect(useUpdateStore.getState().update).toBe(fakeUpdate)
  })

  it('calls check() and resets to idle when no update available', async () => {
    mockedCheck.mockResolvedValue(null)

    const useUpdateChecker = await loadHook()
    renderHook(() => useUpdateChecker())

    await waitFor(() => {
      // After check returns null, dismiss is called which resets to idle
      expect(mockedCheck).toHaveBeenCalledOnce()
    })

    // Status should be idle (dismiss resets it)
    await waitFor(() => {
      expect(useUpdateStore.getState().status).toBe('idle')
    })
    expect(useUpdateStore.getState().update).toBeNull()
  })

  it('calls check() and resets via dismiss when check() throws', async () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    mockedCheck.mockRejectedValue(new Error('Network failure'))

    const useUpdateChecker = await loadHook()
    renderHook(() => useUpdateChecker())

    await waitFor(() => {
      expect(consoleSpy).toHaveBeenCalled()
    })

    // After error, dismiss is called which resets to idle
    expect(useUpdateStore.getState().status).toBe('idle')
    consoleSpy.mockRestore()
  })

  it('does NOT call check() if status is not idle (StrictMode guard)', async () => {
    useUpdateStore.setState({ status: 'checking' })
    mockedCheck.mockResolvedValue(null)

    const useUpdateChecker = await loadHook()
    renderHook(() => useUpdateChecker())

    // Give it time to potentially call check()
    await new Promise((r) => setTimeout(r, 50))
    expect(mockedCheck).not.toHaveBeenCalled()
  })

  it('logs error to console.error when check() fails', async () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
    mockedCheck.mockRejectedValue(new Error('DNS failure'))

    const useUpdateChecker = await loadHook()
    renderHook(() => useUpdateChecker())

    await waitFor(() => {
      expect(consoleSpy).toHaveBeenCalledWith(
        'Update check failed:',
        expect.any(Error),
      )
    })
    consoleSpy.mockRestore()
  })
})
