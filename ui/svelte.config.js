import preprocess from 'svelte-preprocess';
import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  compilerOptions: { hydratable: true },
  adapter: adapter(),
  preprocess: preprocess(),
  ssr: false,
};

export default config;
