import { describe, it, expect, beforeEach } from 'vitest'
import { useDialogStore, DIALOGS, DialogName } from '../dialogStore'

describe('dialogStore', () => {
  beforeEach(() => {
    // Reset store state
    const { closeDialog } = useDialogStore.getState()
    closeDialog()
  })

  it('opens dialog', () => {
    const { openDialog } = useDialogStore.getState()
    
    openDialog(DIALOGS.SFTP_SITE_MANAGER)
    
    expect(useDialogStore.getState().activeDialog).toBe('sftpSiteManager')
  })

  it('closes dialog', () => {
    const { openDialog, closeDialog } = useDialogStore.getState()
    
    openDialog(DIALOGS.SFTP_SITE_MANAGER)
    closeDialog()
    
    expect(useDialogStore.getState().activeDialog).toBeNull()
  })

  it('opens different dialog types', () => {
    const { openDialog } = useDialogStore.getState()
    
    const dialogTypes: DialogName[] = [
      DIALOGS.SFTP_SITE_MANAGER,
      DIALOGS.AI_SETTINGS,
      DIALOGS.COMMAND_PALETTE,
      DIALOGS.INPUT,
    ]
    
    dialogTypes.forEach((type) => {
      openDialog(type)
      expect(useDialogStore.getState().activeDialog).toBe(type)
    })
  })

  it('passes dialog props', () => {
    const { openDialog } = useDialogStore.getState()
    const testProps = { title: 'Test Dialog', message: 'Hello' }
    
    openDialog(DIALOGS.INPUT, testProps)
    
    expect(useDialogStore.getState().activeDialog).toBe('input')
    expect(useDialogStore.getState().dialogProps).toEqual(testProps)
  })

  it('clears props when closing', () => {
    const { openDialog, closeDialog } = useDialogStore.getState()
    
    openDialog(DIALOGS.INPUT, { test: 'data' })
    closeDialog()
    
    expect(useDialogStore.getState().dialogProps).toBeNull()
  })
})
