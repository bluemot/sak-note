import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  // Prevent Vite from clearing the screen
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  // Make Tauri work
  envPrefix: ['VITE_', 'TAURI_'],
  // Ensure relative paths for Tauri WebView
  base: './',
  build: {
    // Tauri supports es2021
    target: ['es2021', 'chrome100', 'safari13'],
    // Don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    // Produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
  },
})
