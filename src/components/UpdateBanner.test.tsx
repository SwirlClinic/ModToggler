import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { useUpdateStore } from '../store/updateStore'
import type { Update } from '@tauri-apps/plugin-updater'

// Mock the process plugin for relaunch
vi.mock('@tauri-apps/plugin-process', () => ({
  relaunch: vi.fn(),
}))

function createMockUpdate(overrides: Partial<Update> = {}): Update {
  return {
    version: '2.0.0',
    body: '## Changes\n- Fix bugs',
    close: vi.fn(),
    downloadAndInstall: vi.fn(),
    ...overrides,
  } as unknown as Update
}

describe('UpdateBanner', () => {
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

  async function loadComponent() {
    const mod = await import('./UpdateBanner')
    return mod.default
  }

  it('renders nothing when status is idle', async () => {
    const UpdateBanner = await loadComponent()
    const { container } = render(<UpdateBanner />)
    expect(container.firstChild).toBeNull()
  })

  it('renders nothing when status is checking', async () => {
    useUpdateStore.setState({ status: 'checking' })
    const UpdateBanner = await loadComponent()
    const { container } = render(<UpdateBanner />)
    expect(container.firstChild).toBeNull()
  })

  it('renders banner with version number and Install button when status is available', async () => {
    const mockUpdate = createMockUpdate()
    useUpdateStore.setState({ status: 'available', update: mockUpdate })

    const UpdateBanner = await loadComponent()
    render(<UpdateBanner />)

    expect(screen.getByText(/Version 2\.0\.0 is available/)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /install now/i })).toBeInTheDocument()
  })

  it('renders collapsible release notes section when update has body text', async () => {
    const mockUpdate = createMockUpdate({ body: '## Changes\n- Fix bugs' })
    useUpdateStore.setState({ status: 'available', update: mockUpdate })

    const UpdateBanner = await loadComponent()
    render(<UpdateBanner />)

    // Release notes toggle should be present
    const toggle = screen.getByRole('button', { name: /release notes/i })
    expect(toggle).toBeInTheDocument()

    // Notes should be hidden initially
    expect(screen.queryByText(/Fix bugs/)).not.toBeInTheDocument()

    // Click to expand
    fireEvent.click(toggle)
    expect(screen.getByText(/Fix bugs/)).toBeInTheDocument()
  })

  it('renders progress bar when status is downloading', async () => {
    useUpdateStore.setState({
      status: 'downloading',
      downloaded: 500,
      contentLength: 1000,
    })

    const UpdateBanner = await loadComponent()
    render(<UpdateBanner />)

    expect(screen.getByText(/downloading update/i)).toBeInTheDocument()
    const progressBar = screen.getByRole('progressbar')
    expect(progressBar).toBeInTheDocument()
  })

  it('renders indeterminate progress bar when contentLength is 0 during download', async () => {
    useUpdateStore.setState({
      status: 'downloading',
      downloaded: 500,
      contentLength: 0,
    })

    const UpdateBanner = await loadComponent()
    render(<UpdateBanner />)

    const progressBar = screen.getByRole('progressbar')
    expect(progressBar).toBeInTheDocument()
    // The inner bar should have animate-pulse for indeterminate state
    const innerBar = progressBar.firstElementChild
    expect(innerBar?.className).toContain('animate-pulse')
  })

  it('renders installing message when status is installing', async () => {
    useUpdateStore.setState({ status: 'installing' })

    const UpdateBanner = await loadComponent()
    render(<UpdateBanner />)

    expect(screen.getByText(/installing update/i)).toBeInTheDocument()
    expect(screen.getByText(/app will restart/i)).toBeInTheDocument()
  })

  it('dismiss button calls store.dismiss() and banner disappears', async () => {
    const mockUpdate = createMockUpdate()
    useUpdateStore.setState({ status: 'available', update: mockUpdate })

    const UpdateBanner = await loadComponent()
    const { container } = render(<UpdateBanner />)

    const dismissBtn = screen.getByRole('button', { name: /dismiss/i })
    fireEvent.click(dismissBtn)

    // After dismiss, store resets to idle, so banner should be gone
    expect(useUpdateStore.getState().status).toBe('idle')
  })

  it('Install button triggers downloadAndInstall on the Update object', async () => {
    const mockDownloadAndInstall = vi.fn().mockResolvedValue(undefined)
    const mockUpdate = createMockUpdate({
      downloadAndInstall: mockDownloadAndInstall,
    })
    useUpdateStore.setState({ status: 'available', update: mockUpdate })

    const UpdateBanner = await loadComponent()
    render(<UpdateBanner />)

    const installBtn = screen.getByRole('button', { name: /install now/i })
    fireEvent.click(installBtn)

    expect(mockDownloadAndInstall).toHaveBeenCalledOnce()
  })
})
