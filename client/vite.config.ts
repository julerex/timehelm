import { defineConfig } from 'vite';

// Note: `make dev-client` runs Vite from `client/`, so the proxy config must live here
// (the repo-root `vite.config.ts` is not picked up when running inside `client/`).
export default defineConfig({
    server: {
        proxy: {
            '/auth': {
                target: 'http://localhost:8080',
                changeOrigin: true
            },
            '/ws': {
                target: 'ws://localhost:8080',
                ws: true
            }
        }
    },
    build: {
        outDir: 'dist'
    }
});


