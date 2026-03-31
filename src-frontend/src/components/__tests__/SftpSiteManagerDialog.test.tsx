import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { SftpSiteManagerDialog } from '../SftpSiteManagerDialog'

describe('SftpSiteManagerDialog', () => {
  const mockOnClose = vi.fn()
  const mockOnConnect = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
    // Reset localStorage mock
    localStorage.clear()
  })

  it('renders empty state when no sites', () => {
    render(
      <SftpSiteManagerDialog
        isOpen={true}
        onClose={mockOnClose}
        onConnect={mockOnConnect}
      />
    )

    expect(screen.getByText('SFTP Site Manager')).toBeInTheDocument()
    expect(screen.getByText('No sites configured')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Add Site/i })).toBeInTheDocument()
  })

  it('opens add site form when clicking Add Site', async () => {
    render(
      <SftpSiteManagerDialog
        isOpen={true}
        onClose={mockOnClose}
        onConnect={mockOnConnect}
      />
    )

    fireEvent.click(screen.getByRole('button', { name: /Add Site/i }))

    await waitFor(() => {
      // Check form labels are present
      expect(screen.getByText(/Site Name/i)).toBeInTheDocument()
      expect(screen.getByText(/Host/i)).toBeInTheDocument()
      expect(screen.getByText(/Port/i)).toBeInTheDocument()
      expect(screen.getByText(/Username/i)).toBeInTheDocument()
      // Check form inputs are present
      expect(document.querySelector('input[type="text"]')).toBeInTheDocument()
      expect(document.querySelector('input[type="number"]')).toBeInTheDocument()
      expect(document.querySelector('input[type="password"]')).toBeInTheDocument()
    })
  })

  it('saves site to localStorage', async () => {
    render(
      <SftpSiteManagerDialog
        isOpen={true}
        onClose={mockOnClose}
        onConnect={mockOnConnect}
      />
    )

    // Click Add Site
    fireEvent.click(screen.getByRole('button', { name: /Add Site/i }))

    // Get all text inputs and fill the form
    await waitFor(() => {
      const textInputs = document.querySelectorAll('input[type="text"]')
      const numberInput = document.querySelector('input[type="number"]')
      const passwordInput = document.querySelector('input[type="password"]')

      expect(textInputs.length).toBeGreaterThanOrEqual(2)
      
      // Fill Site Name
      fireEvent.change(textInputs[0], {
        target: { value: 'Test Server' },
      })
      // Fill Host
      fireEvent.change(textInputs[1], {
        target: { value: '192.168.1.100' },
      })
      // Fill Port
      if (numberInput) {
        fireEvent.change(numberInput, {
          target: { value: '22' },
        })
      }
      // Fill Username
      if (textInputs[2]) {
        fireEvent.change(textInputs[2], {
          target: { value: 'admin' },
        })
      }
      // Fill Password
      if (passwordInput) {
        fireEvent.change(passwordInput, {
          target: { value: 'password123' },
        })
      }
    })

    // Save - click the Add Site button in the footer
    const footerButtons = screen.getAllByRole('button')
    const addButton = footerButtons.find(btn => btn.textContent === 'Add Site')
    if (addButton) {
      fireEvent.click(addButton)
    }

    // Verify localStorage was called
    await waitFor(() => {
      expect(localStorage.setItem).toHaveBeenCalled()
    })
  })

  it('does not render when isOpen is false', () => {
    render(
      <SftpSiteManagerDialog
        isOpen={false}
        onClose={mockOnClose}
        onConnect={mockOnConnect}
      />
    )

    expect(screen.queryByText('SFTP Site Manager')).not.toBeInTheDocument()
  })

  it('closes on Escape key', async () => {
    render(
      <SftpSiteManagerDialog
        isOpen={true}
        onClose={mockOnClose}
        onConnect={mockOnConnect}
      />
    )

    fireEvent.keyDown(window, { key: 'Escape' })

    await waitFor(() => {
      expect(mockOnClose).toHaveBeenCalled()
    })
  })

  it('closes when clicking overlay', async () => {
    render(
      <SftpSiteManagerDialog
        isOpen={true}
        onClose={mockOnClose}
        onConnect={mockOnConnect}
      />
    )

    const overlay = screen.getByText('SFTP Site Manager').closest('.sftp-dialog-overlay')
    expect(overlay).toBeInTheDocument()
    
    if (overlay) {
      fireEvent.click(overlay)
    }

    await waitFor(() => {
      expect(mockOnClose).toHaveBeenCalled()
    })
  })
})