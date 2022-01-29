import preprocess from 'svelte-preprocess';
import wasmPack from 'vite-plugin-wasm-pack';
import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://github.com/sveltejs/svelte-preprocess
  // for more information about preprocessors
  preprocess: preprocess(),

  kit: {
    adapter: adapter(),
    target: '#svelte',

    vite: {
      plugins: [wasmPack([], ['podreplay_lib_wasm'])],
      server: {
        fs: {
          allow: ['./target/wasm-pack'],
        },
        proxy: {
          '/replay': 'http://localhost:8080',
          '/summary': 'http://localhost:8080',
        },
      },
    },
  },
};

export default config;
