import { defineConfig, devices } from '@playwright/test'

// Check if running Tauri E2E tests
const isTauriTest = process.env.TEST_TYPE === 'tauri'

export default defineConfig({
  testDir: './tests/e2e/playwright',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'list',
  use: {
    baseURL: isTauriTest ? undefined : 'http://localhost:5173',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: isTauriTest ? 'tauri' : 'chromium',
      use: isTauriTest
        ? {
            // Tauri E2E mode - test the actual built app
            headless: false,
            viewport: { width: 1400, height: 900 },
            launchOptions: {
              executablePath: './src-tauri/target/release/sak-editor',
              env: {
                ...process.env,
                GSK_RENDERER: 'cairo',
                LIBGL_ALWAYS_SOFTWARE: '1',
                WEBKIT_DISABLE_COMPOSITING_MODE: '1',
              },
            },
          }
        : {
            // Browser mode - test via Vite dev server
            ...devices['Desktop Chrome'],
            viewport: { width: 1400, height: 900 }
          },
    },
  ],
  // Only start web server for browser tests
  webServer: isTauriTest
    ? undefined
    : {
        command: 'cd src-frontend && npm run dev',
        url: 'http://localhost:5173',
        reuseExistingServer: !process.env.CI,
        timeout: 120000,
      },
})
