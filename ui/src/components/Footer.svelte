<script lang="ts">
  const poweredBy = [
    { href: 'https://www.rust-lang.org/', name: 'Rust' },
    { href: 'https://svelte.dev/', name: 'Svelte' },
    { href: 'https://litestream.io/', name: 'Litestream' },
    { href: 'https://fly.io/', name: 'Fly.io' },
    { href: 'https://ikonate.com/', name: 'Iconate' },
  ];
  poweredBy.sort(() => Math.random() - 0.5);
</script>

<footer>
  <p>
    PodReplay is a coffee table project by <a href="https://esimmler.com">Erik Simmler</a>.
  </p>
  <p>
    Powered by
    {#each poweredBy as link (link.name)}
      <a href={link.href}>{link.name}</a>,
    {/each}and more.
  </p>
  <p>
    While this app is &copy; 2021-{new Date().getFullYear()} Erik Simmler, no claims are made to the
    contents of any replayed podcasts.
  </p>
  <p>
    I make no promises or warranties, but you can <a href="https://github.com/tgecho/esimmler.com"
      >file an issue on GitHub</a
    >
    or
    <a href="mailto:erik@podreplay.com">contact me</a>
  </p>
</footer>

<style>
  footer {
    color: #333;
    font-size: 0.9em;
    text-align: center;
    position: absolute;
    bottom: 0.5em;
    left: 0.5em;
    right: 0.5em;
  }
  p {
    margin: 0.2em 0;
  }
</style>
