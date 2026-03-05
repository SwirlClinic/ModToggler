import { useState } from 'react'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'

export interface FileMapping {
  fileName: string
  sourcePath: string
  destinationPath: string
  selected: boolean
}

interface FileMapTableProps {
  files: FileMapping[]
  onChange: (files: FileMapping[]) => void
  showCheckboxes?: boolean
}

export default function FileMapTable({
  files,
  onChange,
  showCheckboxes = false,
}: FileMapTableProps) {
  const [bulkPath, setBulkPath] = useState('')

  const allSelected = files.length > 0 && files.every((f) => f.selected)
  const someSelected = files.some((f) => f.selected)

  function handleSelectAll() {
    const next = !allSelected
    onChange(files.map((f) => ({ ...f, selected: next })))
  }

  function handleSelectOne(index: number) {
    const updated = [...files]
    updated[index] = { ...updated[index], selected: !updated[index].selected }
    onChange(updated)
  }

  function handleDestinationChange(index: number, value: string) {
    const updated = [...files]
    updated[index] = { ...updated[index], destinationPath: value }
    onChange(updated)
  }

  function handleBulkApply() {
    if (!bulkPath.trim()) return
    onChange(
      files.map((f) =>
        f.selected ? { ...f, destinationPath: bulkPath.trim() } : f,
      ),
    )
    setBulkPath('')
  }

  return (
    <div className="space-y-2">
      {/* Bulk edit bar */}
      {showCheckboxes && someSelected && (
        <div className="flex items-center gap-2">
          <Input
            value={bulkPath}
            onChange={(e) => setBulkPath(e.target.value)}
            placeholder="Set path for selected..."
            className="h-7 text-xs flex-1"
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleBulkApply()
            }}
          />
          <Button size="sm" variant="outline" className="h-7 text-xs" onClick={handleBulkApply}>
            Apply
          </Button>
        </div>
      )}

      {/* Table header */}
      <div className="flex items-center gap-2 text-[10px] font-semibold text-muted-foreground uppercase tracking-wider px-1">
        {showCheckboxes && (
          <Checkbox
            checked={allSelected}
            onCheckedChange={handleSelectAll}
            className="h-3.5 w-3.5"
          />
        )}
        <span className="flex-1 min-w-0">File</span>
        <span className="w-40 shrink-0">Destination</span>
      </div>

      {/* File rows */}
      <div className="max-h-52 overflow-y-auto rounded-md bg-muted/20 divide-y divide-border/40">
        {files.map((f, i) => (
          <div key={i} className="flex items-center gap-2 px-1 py-1">
            {showCheckboxes && (
              <Checkbox
                checked={f.selected}
                onCheckedChange={() => handleSelectOne(i)}
                className="h-3.5 w-3.5 shrink-0"
              />
            )}
            <span
              className="flex-1 min-w-0 text-xs text-muted-foreground font-mono truncate"
              title={f.fileName}
            >
              {f.fileName}
            </span>
            <Input
              value={f.destinationPath}
              onChange={(e) => handleDestinationChange(i, e.target.value)}
              className="w-40 h-6 text-xs shrink-0"
            />
          </div>
        ))}
      </div>

      {files.length === 0 && (
        <p className="text-xs text-muted-foreground italic text-center py-3">
          No files selected
        </p>
      )}
    </div>
  )
}
