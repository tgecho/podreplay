<script lang="ts">
  import { derived, type Readable } from 'svelte/store';
  import { sortedQueryString, type State } from '../util/state';

  export let state: Readable<State>;
  export let feedUri: string;
  const url = derived(state, (s) => {
    const queryString = sortedQueryString({ ...s, uri: feedUri });
    return `${location.origin}/replay?${queryString}`;
  });

  let copied = false;
  const copy = navigator.clipboard?.writeText
    ? function copy() {
        navigator.clipboard.writeText($url).then(() => {
          copied = true;
        });
      }
    : null;
  const share =
    'share' in navigator
      ? function share() {
          navigator.share({ url: $url });
        }
      : null;
</script>

<div>
  <a href={$url} target="replay" title={`Feed URL: ${$url}`} class="subscribe icon">Subscribe </a>
  {#if copy}
    <a
      on:click|preventDefault={copy}
      on:mouseenter={() => (copied = false)}
      target="replay"
      title="Copy feed URL to clipboard"
      href={$url}
      class="icon copy"
      class:copied
    >
      Copy Feed URL
    </a>
  {/if}
  {#if share}
    <a
      on:click|preventDefault={share}
      target="replay"
      title="Share feed URL"
      href={$url}
      class="icon share"
    >
      Share Feed URL
    </a>
  {/if}
</div>

<style>
  div {
    flex: 1 1 10em;
    background: #fffcea;
    border: 1px dotted var(--accent-fg-color);
    border-radius: 0.2em;
    display: flex;
    flex-wrap: wrap;
    align-content: flex-start;
    padding: 0.5em;
    gap: 0.4em;
  }
  .icon::before {
    content: ' ';
    display: inline-block;
    background: no-repeat center;
    background-size: contain;
    height: 2em;
    width: 2em;
    margin-top: -2px;
    vertical-align: middle;
  }

  .subscribe::before {
    background-image: url(/rss.svg);
  }
  .copy::before {
    background-image: url(/copy.svg);
    background-size: 1.6em;
    margin-right: 1px;
  }
  .copied::before {
    background-image: url(/ok-circle.svg);
  }
  .share::before {
    background-image: url(/share-ios.svg);
    background-size: 1.7em;
  }
</style>
