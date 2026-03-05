import { useState } from 'react'
import { AlertTriangle, X } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { useIntegrityScan } from '../hooks/useIntegrityScan'

export default function IntegrityAlert() {
  const [dismissed, setDismissed] = useState(false)
  const { data: scan } = useIntegrityScan()

  // Nothing to show: loading, dismissed, or clean scan
  if (dismissed || !scan) return null

  const hasIssues =
    scan.missing_from_game.length > 0 ||
    scan.missing_from_staging.length > 0 ||
    scan.incomplete_journals.length > 0

  if (!hasIssues) return null

  const issues: string[] = []

  if (scan.incomplete_journals.length > 0) {
    issues.push(
      `${scan.incomplete_journals.length} incomplete toggle operation(s) detected — some mods may be in an inconsistent state.`
    )
  }
  if (scan.missing_from_game.length > 0) {
    const names = scan.missing_from_game.map((m) => m.name).join(', ')
    issues.push(`Enabled mod(s) have files missing from game directory: ${names}`)
  }
  if (scan.missing_from_staging.length > 0) {
    const names = scan.missing_from_staging.map((m) => m.name).join(', ')
    issues.push(`Disabled mod(s) have files missing from staging: ${names}`)
  }

  return (
    <div
      role="alert"
      className="mx-4 mt-3 flex items-start gap-3 rounded-md border border-yellow-500/40 bg-yellow-500/10 p-3 text-sm"
    >
      <AlertTriangle className="h-4 w-4 text-yellow-500 mt-0.5 shrink-0" />
      <div className="flex-1 space-y-1">
        {issues.map((issue, i) => (
          <p key={i} className="text-yellow-200">{issue}</p>
        ))}
      </div>
      <Button
        variant="ghost"
        size="icon"
        className="h-6 w-6 shrink-0 text-muted-foreground"
        onClick={() => setDismissed(true)}
        aria-label="Dismiss integrity alert"
      >
        <X className="h-3.5 w-3.5" />
      </Button>
    </div>
  )
}
