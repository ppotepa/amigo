import { defineConfig } from 'vite';

export default defineConfig({
  root: '.',
  publicDir: 'public',
  server: {
    host: '127.0.0.1',
    port: 4191,
  },
  preview: {
    host: '127.0.0.1',
    port: 4191,
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      input: 'strokes.html',
    },
  },
});
