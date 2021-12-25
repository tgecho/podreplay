import preprocess from 'svelte-preprocess';
import wasmPack from 'vite-plugin-wasm-pack';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://github.com/sveltejs/svelte-preprocess
  // for more information about preprocessors
  preprocess: preprocess(),

  kit: {
    ssr: false,
    // hydrate the <div id="svelte"> element in src/app.html
    target: '#svelte',

    vite: {
      plugins: [wasmPack([], ['podreplay_lib_wasm'])],
      server: {
        fs: {
          allow: ['./target/wasm-pack'],
        },
        proxy: {
          '/replay': 'http://localhost:3100',
          '/summary': 'http://localhost:3100',
        },
      },
    },
  },
};

export default config;
