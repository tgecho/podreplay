<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { browser } from '$app/env';
  import { writable } from 'svelte/store';
  import { queryToState, sortedQueryString, State } from '../util/state';

  function localState() {
    return writable<State>(queryToState($page.url.searchParams));
  }

  export let state = localState();
  export let uri = $state?.uri ?? '';

  // We let the noscript for go directly to the /replay endpoint so a valid replay feed can be generated without javascript.
  const action = '/replay';
  // But we override and goto() the /preview endpoint when javascript is available.
  const jsAction = '/preview';

  const handleSubmit = () => {
    $state.uri = uri;
    if (location.pathname !== action) {
      goto(`${jsAction}?${sortedQueryString($state)}`);
    }
  };
</script>

<form {action} on:submit|preventDefault={handleSubmit}>
  <input
    name="uri"
    type="url"
    bind:value={uri}
    required={true}
    placeholder="Enter a URL such as https://example.com/feed"
  />
  <noscript>
    <input type="date" name="start" required={true} />
    <input type="hidden" name="rule" value="1w" />
  </noscript>
  <button disabled={browser && !uri?.trim()}>Load Podcast</button>
</form>

<style>
  form {
    width: 100%;
    display: flex;
    align-items: stretch;
    flex: 1 1 30em;
  }
  input {
    font-size: 1em;
    flex: 1 1 100%;
    padding: 0.35em;
    color: var(--main-fg-color);
    margin: 0;
  }
  button {
    flex: 1 1 7.5em;
    white-space: nowrap;
    background: var(--accent-bg-color);
    color: var(--main-fg-color);
    border: 1px solid var(--main-fg-color);
    padding: 0 0.5em;
    cursor: pointer;
    margin: 0 0 0 -1px;
    font-size: 0.8em;
  }
</style>
