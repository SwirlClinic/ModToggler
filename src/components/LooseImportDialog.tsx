import { useState, useEffect } from 'react'
import { Loader2, FolderOpen } from 'lucide-react'
import { open } from '@tauri-apps/plugin-dialog'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import FileMapTable, { type FileMapping } from './FileMapTable'
import { useImportLooseFiles, useImportLooseZip } from '../hooks/useMods'

interface LooseImportDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  gameId: number
  zipPath?: string
  /** Pre-extracted file names from the zip (for zip flow) */
  zipFileNames?: string[]
}

function nameFromZip(zipPath: string): string {
  const filename = zipPath.split(/[/\\]/).pop() ?? ''
  return filename.replace(/\.zip$/i, '').replace(/[-_]/g, ' ')
}

function fileNameFromPath(path: string): string {
  return path.split(/[/\\]/).pop() ?? path
}

export default function LooseImportDialog({
  open: isOpen,
  onOpenChange,
  gameId,
  zipPath,
  zipFileNames,
}: LooseImportDialogProps) {
  const [modName, setModName] = useState('')
  const [files, setFiles] = useState<FileMapping[]>([])

  const importLoose = useImportLooseFiles()
  const importZip = useImportLooseZip()

  const isZipFlow = !!zipPath
  const isPending = importLoose.isPending || importZip.isPending

  // Reset state when dialog opens
  useEffect(() => {
    if (isOpen) {
      if (isZipFlow && zipPath) {
        setModName(nameFromZip(zipPath))
        // Pre-populate with zip file names if provided
        if (zipFileNames && zipFileNames.length > 0) {
          setFiles(
            zipFileNames.map((name) => ({
              fileName: name,
              sourcePath: name,
              destinationPath: '/',
              selected: true,
            })),
          )
        } else {
          setFiles([])
        }
      } else {
        setModName('')
        setFiles([])
      }
    }
  }, [isOpen, zipPath, zipFileNames, isZipFlow])

  async function handlePickFiles() {
    const selected = await open({
      multiple: true,
      title: 'Select mod files',
    })
    if (selected && Array.isArray(selected) && selected.length > 0) {
      setFiles(
        selected.map((path) => ({
          fileName: fileNameFromPath(path),
          sourcePath: path,
          destinationPath: '/',
          selected: true,
        })),
      )
    }
  }

  function handleImport() {
    if (!modName.trim() || files.length === 0) return

    if (isZipFlow && zipPath) {
      const selectedFiles = files
        .filter((f) => f.selected)
        .map((f) => ({
          source_path: f.sourcePath,
          destination_path: f.destinationPath,
          file_name: f.fileName,
        }))
      if (selectedFiles.length === 0) return
      importZip.mutate(
        {
          gameId,
          zipPath,
          modName: modName.trim(),
          selectedFiles,
        },
        { onSuccess: () => onOpenChange(false) },
      )
    } else {
      const looseFiles = files.map((f) => ({
        source_path: f.sourcePath,
        destination_path: f.destinationPath,
        file_name: f.fileName,
      }))
      importLoose.mutate(
        {
          gameId,
          modName: modName.trim(),
          files: looseFiles,
        },
        { onSuccess: () => onOpenChange(false) },
      )
    }
  }

  const selectedCount = files.filter((f) => f.selected).length

  return (
    <Dialog open={isOpen} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>
            {isZipFlow ? 'Import Loose Files from Zip' : 'Import Loose Files'}
          </DialogTitle>
          <DialogDescription>
            {isZipFlow
              ? 'Select files to import and set their destination paths relative to the game directory.'
              : 'Pick files and map them to destination paths relative to the game directory.'}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-3">
          {/* Mod name input */}
          <div>
            <label className="text-xs font-medium text-muted-foreground mb-1 block">
              Mod Name
            </label>
            <Input
              value={modName}
              onChange={(e) => setModName(e.target.value)}
              placeholder="My Mod"
              autoFocus={!isZipFlow}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && files.length > 0) handleImport()
              }}
            />
          </div>

          {/* Manual file picker button */}
          {!isZipFlow && (
            <Button variant="outline" size="sm" onClick={handlePickFiles} className="w-full">
              <FolderOpen className="h-4 w-4" />
              {files.length > 0 ? `${files.length} files selected - Pick more` : 'Pick Files'}
            </Button>
          )}

          {/* Zip path display */}
          {isZipFlow && zipPath && (
            <div>
              <label className="text-xs font-medium text-muted-foreground mb-1 block">
                Archive
              </label>
              <p className="text-xs text-muted-foreground font-mono truncate" title={zipPath}>
                {zipPath}
              </p>
            </div>
          )}

          {/* File mapping table */}
          {files.length > 0 && (
            <FileMapTable
              files={files}
              onChange={setFiles}
              showCheckboxes={isZipFlow}
            />
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleImport}
            disabled={isPending || !modName.trim() || files.length === 0 || (isZipFlow && selectedCount === 0)}
          >
            {isPending && <Loader2 className="h-4 w-4 animate-spin" />}
            Import{isZipFlow && selectedCount > 0 ? ` (${selectedCount})` : ''}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
