import { describe, it, expect, vi } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import Toolbar from '../components/Toolbar'

describe('Toolbar Component', () => {
  const mockProps = {
    onOpenFile: vi.fn(),
    onCloseFile: vi.fn(),
    onToggleView: vi.fn(),
    viewMode: 'text' as const,
    hasFile: false,
    isLoading: false
  }

  it('renders Open button when no file is loaded', () => {
    render(<Toolbar {...mockProps} />)
    expect(screen.getByText('📂 Open')).toBeInTheDocument()
  })

  it('renders Close button when file is loaded', () => {
    render(<Toolbar {...mockProps} hasFile={true} />)
    expect(screen.getByText('❌ Close')).toBeInTheDocument()
  })

  it('renders View toggle when file is loaded', () => {
    render(<Toolbar {...mockProps} hasFile={true} />)
    expect(screen.getByText('🔍 Hex View')).toBeInTheDocument()
  })

  it('calls onOpenFile when Open button is clicked', () => {
    render(<Toolbar {...mockProps} />)
    fireEvent.click(screen.getByText('📂 Open'))
    expect(mockProps.onOpenFile).toHaveBeenCalled()
  })

  it('calls onCloseFile when Close button is clicked', () => {
    render(<Toolbar {...mockProps} hasFile={true} />)
    fireEvent.click(screen.getByText('❌ Close'))
    expect(mockProps.onCloseFile).toHaveBeenCalled()
  })

  it('calls onToggleView when View button is clicked', () => {
    render(<Toolbar {...mockProps} hasFile={true} />)
    fireEvent.click(screen.getByText('🔍 Hex View'))
    expect(mockProps.onToggleView).toHaveBeenCalled()
  })

  it('shows Search input when search button is clicked', () => {
    render(<Toolbar {...mockProps} hasFile={true} />)
    fireEvent.click(screen.getByText('🔎 Search'))
    expect(screen.getByPlaceholderText('Search...')).toBeInTheDocument()
  })

  it('disables Open button when loading', () => {
    render(<Toolbar {...mockProps} isLoading={true} />)
    expect(screen.getByText('Loading...')).toBeDisabled()
  })

  it('switches between Text and Hex view labels', () => {
    const { rerender } = render(<Toolbar {...mockProps} hasFile={true} viewMode="text" />)
    expect(screen.getByText('🔍 Hex View')).toBeInTheDocument()
    
    rerender(<Toolbar {...mockProps} hasFile={true} viewMode="hex" />)
    expect(screen.getByText('📝 Text View')).toBeInTheDocument()
  })
})
