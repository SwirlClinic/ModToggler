import { create } from 'zustand'
import type { Update } from '@tauri-apps/plugin-updater'

export type UpdateStatus = 'idle' | 'checking' | 'available' | 'downloading' | 'installing' | 'error'

interface UpdateStore {
  status: UpdateStatus
  update: Update | null
  downloaded: number
  contentLength: number
  error: string | null

  setChecking: () => void
  setAvailable: (update: Update) => void
  setDownloading: () => void
  addProgress: (chunkLength: number) => void
  setContentLength: (len: number) => void
  setInstalling: () => void
  setError: (msg: string) => void
  dismiss: () => void
}

export const useUpdateStore = create<UpdateStore>((set, get) => ({
  status: 'idle',
  update: null,
  downloaded: 0,
  contentLength: 0,
  error: null,

  setChecking: () => set({ status: 'checking' }),
  setAvailable: (update) => set({ status: 'available', update }),
  setDownloading: () => set({ status: 'downloading', downloaded: 0, contentLength: 0 }),
  addProgress: (chunkLength) => set((s) => ({ downloaded: s.downloaded + chunkLength })),
  setContentLength: (len) => set({ contentLength: len }),
  setInstalling: () => set({ status: 'installing' }),
  setError: (msg) => set({ status: 'error', error: msg }),
  dismiss: () => {
    const { update } = get()
    if (update) update.close()
    set({ status: 'idle', update: null, downloaded: 0, contentLength: 0, error: null })
  },
}))
