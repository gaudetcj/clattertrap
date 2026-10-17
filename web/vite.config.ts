import { defineConfig } from 'vitest/config';

// The page bundles levels/ from the repository root, one directory up.
export default defineConfig({
  server: {
    fs: {
      allow: ['..'],
    },
  },
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts'],
  },
});
