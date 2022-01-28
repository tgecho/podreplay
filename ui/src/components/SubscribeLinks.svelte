<script lang="ts">
  import { derived, Readable } from 'svelte/store';
  import { sortedQueryString, State } from '../util/state';

  export let state: Readable<State>;
  const url = derived(state, (s) => {
    const queryString = sortedQueryString(s);
    return `${location.origin}/replay?${queryString}`;
  });

  let copying: undefined | Promise<unknown>;
  const copy = navigator.clipboard?.writeText
    ? function copy() {
        copying = navigator.clipboard.writeText($url);
      }
    : null;
</script>

<div>
  <a href={$url} target="replay" title={`Feed URL: ${$url}`} class="subscribe-url">Subscribe</a>
  {#if copy}
    <button on:click={copy}>
      {#if copying}
        {#await copying}
          Copying
        {:then _}
          Copied!
        {/await}
      {:else}
        Copy
      {/if}
    </button>
  {/if}
</div>

<style>
  div {
    flex: 1 1 10em;
    margin: 0.75em;
    background: #fff;
    border: 1px dotted var(--accent-fg-color);
    padding: 1em;
  }
</style>
