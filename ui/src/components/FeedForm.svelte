<script lang="ts">
  import { goto } from '$app/navigation';

  export let uri = '';

  const handleSubmit: svelte.JSX.FormEventHandler<HTMLFormElement> = (ev) => {
    const form = ev.currentTarget as HTMLFormElement;
    const params = new URLSearchParams();
    for (const el of Array.from(form.elements) as HTMLInputElement[]) {
      if (el.name && el.value) params.set(el.name, el.value);
    }
    goto(`${form.target}?${params.toString()}`);
  };
</script>

<form action="/" on:submit|preventDefault={handleSubmit}>
  <input name="uri" value={uri} />
  <button>Load</button>
</form>

<style>
  input {
    width: 100%;
  }
</style>
