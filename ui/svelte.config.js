import preprocess from 'svelte-preprocess';
import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  compilerOptions: { hydratable: true },
  preprocess: preprocess(),
  kit: {
    adapter: adapter(),
  },
};

export default config;
