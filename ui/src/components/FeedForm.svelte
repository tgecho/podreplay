<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { writable } from 'svelte/store';
  import { queryToState, sortedQueryString, State } from '../util/state';

  function localState() {
    return writable<State>(queryToState($page.url.searchParams));
  }

  export let action = '/preview';
  export let state = localState();
  export let uri = $state?.uri ?? '';

  const handleSubmit = () => {
    $state.uri = uri;

    if (location.pathname !== action) {
      goto(`${action}?${sortedQueryString($state)}`);
    }
  };
</script>

<form {action} on:submit|preventDefault={handleSubmit}>
  <input name="uri" type="url" bind:value={uri} />
  <button disabled={!uri?.trim()}>Load Podcast</button>
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
    flex: 1 0 7.5em;
    background: var(--accent-bg-color);
    color: var(--main-fg-color);
    border: 1px solid var(--main-fg-color);
    padding: 0 0.1em 0 0;
    cursor: pointer;
    margin: 0 0 0 -1px;
  }
</style>
