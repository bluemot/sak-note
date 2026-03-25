# Testing Strategies for Tauri (React + Rust) Hybrid Architecture

For Tauri hybrid architectures (frontend React + backend Rust), modern automated testing typically adopts a **"Layered Testing" (Test Pyramid)** strategy. Similar to writing low-level system or backend scripts, but using different tools for each layer:

---

## 1. Backend Core Logic Testing (Rust Unit & Integration Tests)

The bottom layer is responsible for system operations in Rust modules (e.g., `file_engine`, `sftp_module` in the project). These can be tested completely without launching the UI.

**How:** Rust has a powerful and elegant built-in testing framework. Simply write test functions alongside your source code, then run `cargo test` in the terminal.

**Concept:** Similar to writing `assert` in C/C++ or using Google Test—directly verifying the correctness of data structures and algorithms with extremely fast execution.

---

## 2. Frontend Component Testing

The UI is also code, and often we can simulate UI generation "in memory" without actually opening a graphical window.

**Tools:** Typically use **Vitest** (ultra-fast testing framework for Vite) combined with **React Testing Library**.

**How:** Write a script to isolate a "Toolbar" or "Sidebar" component, trigger Click events programmatically, then verify that internal state changes correctly.

---

## 3. End-to-End Automation Testing (E2E) — "The Robot That Replaces Manual Clicking"

This is the main event—automation to replace manual clicking! Instead of clicking ourselves, we write scripts for a "program robot" to control the application.

**Tools:** Currently, the most popular and powerful tools are Microsoft's open-source **Playwright**, or **WebDriverIO** (Tauri officially provides integration support).

**How:** These tools actually launch your Tauri desktop application, then execute operations based on your scripts (typically TypeScript or Python) to find elements on screen and interact with them.

### Example Playwright Test Script:

```typescript
// e2e_tests/hex_viewer.spec.ts
import { test, expect } from '@playwright/test';

test('should open file and display hex content', async ({ page }) => {
  // 1. App is launched automatically by the test runner setup
  
  // 2. Locate the open button and simulate a mouse click
  const openBtn = page.locator('#btn-open-local-file');
  await openBtn.click();
  
  // 3. Verify the Hex Viewer container appears on screen
  const hexView = page.locator('.hex-viewer-container');
  await expect(hexView).toBeVisible();
  
  // 4. Check if specific memory address text exists
  const addressLine = page.getByText('00000000');
  await expect(addressLine).toBeVisible();
});
```

---

## 4. Combining CI/CD with AI Assistance

**CI/CD Automation Pipeline:** These tests are typically integrated into version control platforms (e.g., GitHub Actions). When you push code, cloud servers automatically compile and run: `cargo test` → component tests → launch headless window to run Playwright click tests. Code can only be merged after all tests pass.

**The Role of AI:** Even the UI control test scripts above no longer need to be written from scratch. When developing tools like SAK-Editor that integrate LLMs, you can have the language model read the React source code and automatically generate corresponding Playwright test scripts.

---

## Summary

Push logic as low as possible (rely on Rust unit tests for validation), and only use automation scripts like Playwright at the top layer for a few critical UI operation path validations. This is the current standard and modern best practice.
