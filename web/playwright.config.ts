import process from 'node:process';
import { defineConfig } from '@playwright/test';

// Two projects on one built site: the grid check `just ci` runs, and the hero recorder `just hero`
// runs. The server is a preview of the production build, so what is tested is what ships.
export default defineConfig({
  projects: [
    { name: 'e2e', testDir: 'e2e', timeout: 90_000 },
    { name: 'hero', testDir: 'hero', timeout: 600_000 },
  ],
  workers: 1,
  retries: 0,
  reporter: 'list',
  use: {
    baseURL: 'http://localhost:4173',
    browserName: 'chromium',
    viewport: { width: 1280, height: 900 },
    deviceScaleFactor: 1,
  },
  webServer: {
    command: 'npm run serve',
    url: 'http://localhost:4173/arena.html',
    reuseExistingServer: !process.env['CI'],
    timeout: 180_000,
  },
});
