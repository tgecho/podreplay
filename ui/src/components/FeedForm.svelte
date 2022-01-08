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
  <input name="uri" bind:value={uri} />
  <button disabled={!uri?.trim()}>Load</button>
</form>

<style>
  input {
    width: 100%;
  }
</style>
