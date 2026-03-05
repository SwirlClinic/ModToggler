import { useState } from 'react'
import { ChevronDown } from 'lucide-react'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Button } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import SaveProfileDialog from './SaveProfileDialog'
import ManageProfilesDialog from './ManageProfilesDialog'
import { useProfiles, useLoadProfile } from '../hooks/useProfiles'
import { useGameStore } from '../store/gameStore'

interface ProfileDropdownProps {
  gameId: number
}

export default function ProfileDropdown({ gameId }: ProfileDropdownProps) {
  const { data: profiles = [] } = useProfiles(gameId)
  const loadProfile = useLoadProfile()
  const lastLoadedProfileName = useGameStore((s) => s.lastLoadedProfileName)

  const [popoverOpen, setPopoverOpen] = useState(false)
  const [saveOpen, setSaveOpen] = useState(false)
  const [manageOpen, setManageOpen] = useState(false)

  function handleLoadProfile(profileId: number, profileName: string) {
    loadProfile.mutate({ profileId, profileName })
    setPopoverOpen(false)
  }

  function handleSaveClick() {
    setPopoverOpen(false)
    setSaveOpen(true)
  }

  function handleManageClick() {
    setPopoverOpen(false)
    setManageOpen(true)
  }

  return (
    <>
      <Popover open={popoverOpen} onOpenChange={setPopoverOpen}>
        <PopoverTrigger asChild>
          <Button variant="outline" size="sm">
            {lastLoadedProfileName || 'Profiles'}
            <ChevronDown className="h-4 w-4 ml-1" />
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-48 p-1" align="end">
          {profiles.map((profile) => (
            <button
              key={profile.id}
              className="w-full text-left text-sm px-2 py-1.5 rounded-sm hover:bg-accent hover:text-accent-foreground"
              onClick={() => handleLoadProfile(profile.id, profile.name)}
            >
              {profile.name}
            </button>
          ))}

          {profiles.length > 0 && <Separator className="my-1" />}

          <button
            className="w-full text-left text-sm px-2 py-1.5 rounded-sm hover:bg-accent hover:text-accent-foreground"
            onClick={handleSaveClick}
          >
            Save Current...
          </button>
          <button
            className="w-full text-left text-sm px-2 py-1.5 rounded-sm hover:bg-accent hover:text-accent-foreground"
            onClick={handleManageClick}
          >
            Manage Profiles
          </button>
        </PopoverContent>
      </Popover>

      <SaveProfileDialog
        open={saveOpen}
        onOpenChange={setSaveOpen}
        gameId={gameId}
        existingNames={profiles.map((p) => p.name)}
      />

      <ManageProfilesDialog
        open={manageOpen}
        onOpenChange={setManageOpen}
        gameId={gameId}
      />
    </>
  )
}
