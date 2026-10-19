import { defineConfig } from 'vitest/config';

// The pages bundle levels/ and gallery/ from the repository root, one directory up.
export default defineConfig({
  server: {
    fs: {
      allow: ['..'],
    },
  },
  build: {
    rollupOptions: {
      input: {
        index: 'index.html',
        arena: 'arena.html',
        reel: 'reel.html',
        gallery: 'gallery.html',
        leaderboard: 'leaderboard.html',
      },
    },
  },
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts'],
  },
});
