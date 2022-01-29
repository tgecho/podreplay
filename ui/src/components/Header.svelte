<header>
  <h1><a href="/">PodReplay</a></h1>
  <slot />
</header>

<style>
  header {
    display: flex;
    align-items: stretch;
    flex-wrap: wrap;
    gap: 0.5em;
  }
  h1 {
    flex: 0 0 160px;
    font-size: 1.5rem;
    margin: 0.2em auto;
  }
  h1::before {
    content: ' ';
    display: inline-block;
    background: url(/history-heavy.svg);
    background-size: contain;
    height: 1.25em;
    width: 1.25em;
    margin: -2px 0.2em 0 0;
    vertical-align: middle;
  }
</style>
