import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useUpdateStore } from './updateStore'
import type { Update } from '@tauri-apps/plugin-updater'

const initialState = {
  status: 'idle',
  update: null,
  downloaded: 0,
  contentLength: 0,
  error: null,
}

describe('updateStore', () => {
  beforeEach(() =>
    useUpdateStore.setState({
      status: 'idle',
      update: null,
      downloaded: 0,
      contentLength: 0,
      error: null,
    }),
  )

  it('initial state is idle with null update, 0 downloaded, 0 contentLength, null error', () => {
    const state = useUpdateStore.getState()
    expect(state.status).toBe('idle')
    expect(state.update).toBeNull()
    expect(state.downloaded).toBe(0)
    expect(state.contentLength).toBe(0)
    expect(state.error).toBeNull()
  })

  it('setChecking transitions status to checking', () => {
    useUpdateStore.getState().setChecking()
    expect(useUpdateStore.getState().status).toBe('checking')
  })

  it('setAvailable stores the Update object and sets status to available', () => {
    const fakeUpdate = { version: '2.0.0', body: 'notes' } as unknown as Update
    useUpdateStore.getState().setAvailable(fakeUpdate)
    const state = useUpdateStore.getState()
    expect(state.status).toBe('available')
    expect(state.update).toBe(fakeUpdate)
  })

  it('setDownloading resets downloaded/contentLength to 0 and sets status to downloading', () => {
    useUpdateStore.setState({ downloaded: 500, contentLength: 1000 })
    useUpdateStore.getState().setDownloading()
    const state = useUpdateStore.getState()
    expect(state.status).toBe('downloading')
    expect(state.downloaded).toBe(0)
    expect(state.contentLength).toBe(0)
  })

  it('addProgress accumulates chunkLength onto downloaded', () => {
    useUpdateStore.getState().addProgress(100)
    useUpdateStore.getState().addProgress(250)
    expect(useUpdateStore.getState().downloaded).toBe(350)
  })

  it('setContentLength stores the total bytes', () => {
    useUpdateStore.getState().setContentLength(5000)
    expect(useUpdateStore.getState().contentLength).toBe(5000)
  })

  it('setInstalling sets status to installing', () => {
    useUpdateStore.getState().setInstalling()
    expect(useUpdateStore.getState().status).toBe('installing')
  })

  it('setError stores error message and sets status to error', () => {
    useUpdateStore.getState().setError('Network failure')
    const state = useUpdateStore.getState()
    expect(state.status).toBe('error')
    expect(state.error).toBe('Network failure')
  })

  it('dismiss calls update.close() if update exists, then resets all state to initial', () => {
    const closeFn = vi.fn()
    const fakeUpdate = { close: closeFn } as unknown as Update
    useUpdateStore.setState({ status: 'available', update: fakeUpdate })
    useUpdateStore.getState().dismiss()
    expect(closeFn).toHaveBeenCalledOnce()
    const state = useUpdateStore.getState()
    expect(state.status).toBe('idle')
    expect(state.update).toBeNull()
    expect(state.downloaded).toBe(0)
    expect(state.contentLength).toBe(0)
    expect(state.error).toBeNull()
  })

  it('dismiss with null update resets without calling close()', () => {
    useUpdateStore.setState({ status: 'error', error: 'some error' })
    useUpdateStore.getState().dismiss()
    const state = useUpdateStore.getState()
    expect(state.status).toBe('idle')
    expect(state.update).toBeNull()
    expect(state.error).toBeNull()
  })
})
