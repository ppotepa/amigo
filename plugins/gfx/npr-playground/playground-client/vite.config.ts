import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
export default defineConfig({ plugins: [svelte()], base: './', build: { rollupOptions: { output: { entryFileNames: 'client.js', chunkFileNames: '[name].js', assetFileNames: '[name][extname]' } } } });
