<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { queryToState, sortedQueryString } from '../util/state';

  export let uri = '';

  const handleSubmit = (ev: Event) => {
    const form = ev.currentTarget as HTMLFormElement;
    const query = $page.url.searchParams;

    for (const el of Array.from(form.elements) as HTMLInputElement[]) {
      if (el.name && el.value) query.set(el.name, el.value);
    }

    goto(`${form.action}?${sortedQueryString(queryToState(query))}`);
  };
</script>

<form action="/preview" on:submit|preventDefault={handleSubmit}>
  <input name="uri" value={uri} />
  <button>Load</button>
</form>

<style>
  input {
    width: 100%;
  }
</style>
