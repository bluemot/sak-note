// Test setup file
import '@testing-library/jest-dom'
import { vi } from 'vitest'

// Mock Tauri API
vi.mock('@tauri-apps/api/dialog', () => ({
  open: vi.fn()
}))

vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn()
}))
