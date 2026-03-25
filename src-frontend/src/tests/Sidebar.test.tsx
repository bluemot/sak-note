import { describe, it, expect } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import Sidebar from '../components/Sidebar'

describe('Sidebar Component', () => {
  const mockFile = {
    path: '/home/user/test.txt',
    size: 1024,
    chunks: 4,
    chunk_size: 256
  }

  it('renders three tabs', () => {
    render(<Sidebar currentFile={null} />)
    expect(screen.getByText('ℹ️ Info')).toBeInTheDocument()
    expect(screen.getByText('🤖 Chat')).toBeInTheDocument()
    expect(screen.getByText('🎨 Marks')).toBeInTheDocument()
  })

  it('shows "No file open" when no file is loaded', () => {
    render(<Sidebar currentFile={null} />)
    expect(screen.getByText('No file open')).toBeInTheDocument()
  })

  it('displays file info when file is loaded', () => {
    render(<Sidebar currentFile={mockFile} />)
    expect(screen.getByText('test.txt')).toBeInTheDocument()
    expect(screen.getByText('1 KB')).toBeInTheDocument()
    expect(screen.getByText('4')).toBeInTheDocument()
    expect(screen.getByText('256 B')).toBeInTheDocument()
  })

  it('switches between tabs', () => {
    render(<Sidebar currentFile={null} />)
    
    // Click Chat tab
    fireEvent.click(screen.getByText('🤖 Chat'))
    expect(screen.getByText('Open a file to start chatting')).toBeInTheDocument()
    
    // Click Marks tab
    fireEvent.click(screen.getByText('🎨 Marks'))
    expect(screen.getByText('Color highlights will appear here')).toBeInTheDocument()
  })

  it('formats large file sizes correctly', () => {
    const largeFile = {
      ...mockFile,
      size: 1024 * 1024 * 5, // 5 MB
    }
    render(<Sidebar currentFile={largeFile} />)
    expect(screen.getByText('5 MB')).toBeInTheDocument()
  })

  it('shows chat placeholder when file is open but in Chat tab', () => {
    render(<Sidebar currentFile={mockFile} />)
    fireEvent.click(screen.getByText('🤖 Chat'))
    // LlmChat component should render when file is open
  })
})
