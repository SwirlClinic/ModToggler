import { useState } from 'react'
import { Download, ChevronDown, ChevronUp, X } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { relaunch } from '@tauri-apps/plugin-process'
import { useUpdateStore } from '../store/updateStore'

export default function UpdateBanner() {
  const status = useUpdateStore((s) => s.status)
  const update = useUpdateStore((s) => s.update)
  const downloaded = useUpdateStore((s) => s.downloaded)
  const contentLength = useUpdateStore((s) => s.contentLength)
  const dismiss = useUpdateStore((s) => s.dismiss)

  const [notesOpen, setNotesOpen] = useState(false)

  if (status === 'idle' || status === 'checking' || status === 'error') {
    return null
  }

  async function handleInstall() {
    const store = useUpdateStore.getState()
    const currentUpdate = store.update
    if (!currentUpdate) return

    try {
      store.setDownloading()

      await currentUpdate.downloadAndInstall((event) => {
        const s = useUpdateStore.getState()
        if (event.event === 'Started') {
          s.setContentLength(event.data.contentLength ?? 0)
        } else if (event.event === 'Progress') {
          s.addProgress(event.data.chunkLength)
        } else if (event.event === 'Finished') {
          s.setInstalling()
        }
      })

      await relaunch()
    } catch (err) {
      useUpdateStore.getState().setError(String(err))
    }
  }

  if (status === 'downloading') {
    const pct = contentLength > 0 ? (downloaded / contentLength) * 100 : 100

    return (
      <div
        role="alert"
        className="mx-4 mt-3 flex flex-col gap-2 rounded-md border border-blue-500/40 bg-blue-500/10 p-3 text-sm"
      >
        <div className="flex items-center gap-3">
          <Download className="h-4 w-4 text-blue-400 shrink-0" />
          <span className="text-blue-200">Downloading update...</span>
        </div>
        <div
          role="progressbar"
          aria-valuenow={contentLength > 0 ? Math.round(pct) : undefined}
          aria-valuemin={0}
          aria-valuemax={100}
          className="h-1.5 w-full rounded-full bg-blue-500/20 overflow-hidden"
        >
          <div
            className={`h-full rounded-full bg-blue-500 transition-all${contentLength === 0 ? ' animate-pulse' : ''}`}
            style={{ width: `${pct}%` }}
          />
        </div>
      </div>
    )
  }

  if (status === 'installing') {
    return (
      <div
        role="alert"
        className="mx-4 mt-3 flex items-center gap-3 rounded-md border border-blue-500/40 bg-blue-500/10 p-3 text-sm"
      >
        <Download className="h-4 w-4 text-blue-400 shrink-0" />
        <span className="text-blue-200">
          Installing update... The app will restart.
        </span>
      </div>
    )
  }

  // status === 'available'
  return (
    <div
      role="alert"
      className="mx-4 mt-3 flex items-start gap-3 rounded-md border border-blue-500/40 bg-blue-500/10 p-3 text-sm"
    >
      <Download className="h-4 w-4 text-blue-400 mt-0.5 shrink-0" />
      <div className="flex-1 space-y-1">
        <p className="text-blue-200">
          Version {update?.version} is available.{' '}
          <button
            className="underline hover:text-blue-100"
            onClick={handleInstall}
            aria-label="Install now"
          >
            Install now
          </button>
        </p>
        <p className="text-xs text-blue-300/60">
          The app will briefly close to install.
        </p>
        {update?.body && (
          <>
            <button
              className="flex items-center gap-1 text-xs text-blue-300/80 hover:text-blue-200"
              onClick={() => setNotesOpen(!notesOpen)}
              aria-label="Release notes"
            >
              {notesOpen ? (
                <ChevronUp className="h-3 w-3" />
              ) : (
                <ChevronDown className="h-3 w-3" />
              )}
              Release notes
            </button>
            {notesOpen && (
              <pre className="whitespace-pre-wrap text-xs text-blue-300/80 mt-2 max-h-48 overflow-auto">
                {update.body}
              </pre>
            )}
          </>
        )}
      </div>
      <Button
        variant="ghost"
        size="icon"
        className="h-6 w-6 shrink-0 text-muted-foreground"
        onClick={dismiss}
        aria-label="Dismiss update banner"
      >
        <X className="h-3.5 w-3.5" />
      </Button>
    </div>
  )
}
