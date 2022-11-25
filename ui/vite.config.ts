import { sveltekit } from '@sveltejs/kit/vite';
import { searchForWorkspaceRoot } from 'vite';

/** @type {import('vite').UserConfig} */
const config = {
  plugins: [sveltekit()],
  server: {
    port: 3000,
    fs: {
      allow: [searchForWorkspaceRoot(process.cwd()), './target/wasm-pack'],
    },
    proxy: {
      '/replay': 'http://localhost:8080',
      '/summary': 'http://localhost:8080',
    },
  },
};

export default config;
