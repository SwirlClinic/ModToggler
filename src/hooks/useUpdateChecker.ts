import { useEffect } from 'react'
import { check } from '@tauri-apps/plugin-updater'
import { useUpdateStore } from '../store/updateStore'

export function useUpdateChecker() {
  const status = useUpdateStore((s) => s.status)
  const setChecking = useUpdateStore((s) => s.setChecking)
  const setAvailable = useUpdateStore((s) => s.setAvailable)
  const dismiss = useUpdateStore((s) => s.dismiss)

  useEffect(() => {
    if (status !== 'idle') return

    let cancelled = false
    setChecking()

    check()
      .then((update) => {
        if (cancelled) return
        if (update) {
          setAvailable(update)
        } else {
          dismiss()
        }
      })
      .catch((err) => {
        if (cancelled) return
        console.error('Update check failed:', err)
        dismiss()
      })

    return () => {
      cancelled = true
    }
  }, []) // eslint-disable-line react-hooks/exhaustive-deps
}
