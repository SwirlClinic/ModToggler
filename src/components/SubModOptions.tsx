import { Loader2 } from 'lucide-react'
import { useSubMods, useToggleSubMod } from '../hooks/useMods'

interface SubModOptionsProps {
  modId: number
  parentEnabled: boolean
}

export default function SubModOptions({ modId, parentEnabled }: SubModOptionsProps) {
  const { data: subMods = [] } = useSubMods(modId)
  const toggleSubMod = useToggleSubMod()

  if (subMods.length === 0) return null

  return (
    <div className="mt-3">
      <h4 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-2">
        Options
      </h4>
      <div className="space-y-1.5">
        {subMods.map((sub) => (
          <div
            key={sub.id}
            className="flex items-center justify-between rounded-md px-2 py-1.5 bg-muted/30"
          >
            <span className="text-sm truncate mr-2">{sub.name}</span>
            <button
              type="button"
              disabled={!parentEnabled || toggleSubMod.isPending}
              onClick={(e) => {
                e.stopPropagation()
                toggleSubMod.mutate({ subModId: sub.id, enable: !sub.enabled })
              }}
              className={`relative inline-flex h-5 w-9 shrink-0 items-center rounded-full transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring ${
                !parentEnabled
                  ? 'opacity-40 cursor-not-allowed bg-muted'
                  : sub.enabled
                    ? 'bg-emerald-600'
                    : 'bg-muted-foreground/30'
              }`}
              aria-label={`Toggle ${sub.name}`}
            >
              {toggleSubMod.isPending ? (
                <Loader2 className="h-3 w-3 animate-spin mx-auto text-foreground" />
              ) : (
                <span
                  className={`inline-block h-3.5 w-3.5 rounded-full bg-white transition-transform ${
                    sub.enabled ? 'translate-x-4' : 'translate-x-1'
                  }`}
                />
              )}
            </button>
          </div>
        ))}
      </div>
    </div>
  )
}
