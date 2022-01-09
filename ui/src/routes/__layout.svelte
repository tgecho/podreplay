<slot />

<style>
  :global(*) {
    box-sizing: border-box;
  }
  :global(body) {
    --main-bg-color: #fff;
    --accent-bg-color: #eee;
    --main-fg-color: #000;
    --accent-fg-color: #444;

    background: var(--main-bg-color);
    color: var(--main-fg-color);

    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif,
      'Apple Color Emoji', 'Segoe UI Emoji', 'Segoe UI Symbol';
    max-width: 50em;
    margin: auto;
    padding: 1em;
  }
  @media (max-width: 30em) {
    :global(body) {
      padding: 0.5em;
    }
  }
  @media (prefers-color-scheme: dark) {
    :global(body) {
      --main-bg-color: #444;
      --accent-bg-color: #111;
      --main-fg-color: #fff;
      --accent-fg-color: #ccc;
    }
  }
</style>
