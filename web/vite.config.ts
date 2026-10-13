import { defineConfig } from 'vitest/config';

// The fixture levels live in the core crate's tests until milestone 4 moves them to levels/.
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
