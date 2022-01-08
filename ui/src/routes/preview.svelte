<script context="module" lang="ts">
  import { FeedSummary, fetchFeedSummary } from '../util/fetchFeedSummary';
  import type { Load } from '@sveltejs/kit';

  export const load: Load = async ({ url, fetch }) => {
    const uri = url.searchParams.get('uri');
    const feed = uri ? await fetchFeedSummary(uri, fetch) : null;
    return { props: { feed } };
  };
</script>

<script lang="ts">
  import FeedForm from '../components/FeedForm.svelte';
  import ItemPreview from '../components/ItemPreview.svelte';
  import { queryStore, replayUrlStore } from '../util/state';

  export let feed: FeedSummary;
  const start = new Date().toISOString();

  const state = queryStore();
  const replayUrl = replayUrlStore(state, start);

  $: s = $state.interval > 1 ? 's' : '';
</script>

<h1>PodReplay</h1>

<FeedForm uri={$state.uri} />

{#if feed}
  <br /><b>Title:</b>
  {feed.title}
  <br /><b>URI:</b>
  {feed.uri}

  <form target="/preview" on:submit|preventDefault>
    <input type="hidden" name="uri" value={$state.uri} />

    <fieldset>
      Custom Title:
      <input name="title" bind:value={$state.title} placeholder={`${feed.title} (PodReplay)`} />
    </fieldset>

    <fieldset>
      A replayed episode every
      <input type="number" name="interval" bind:value={$state.interval} min={1} max={10} />
    </fieldset>

    <fieldset>
      <label>
        <input type="radio" name="freq" bind:group={$state.freq} value="m" />
        Month{s}
      </label>
      <label>
        <input type="radio" name="freq" bind:group={$state.freq} value="w" />
        Week{s}
      </label>
      <label>
        <input type="radio" name="freq" bind:group={$state.freq} value="d" /> Day{s}
      </label>
    </fieldset>
    {#if $state.freq === 'w'}
      <fieldset>
        On
        <label>
          <input type="checkbox" bind:checked={$state.weekdays.Su} name="weekday-Su" />
          Sunday
        </label>
        <label>
          <input type="checkbox" bind:checked={$state.weekdays.M} name="weekday-M" />
          Monday
        </label>
        <label>
          <input type="checkbox" bind:checked={$state.weekdays.Tu} name="weekday-Tu" />
          Tuesday
        </label>
        <label>
          <input type="checkbox" bind:checked={$state.weekdays.W} name="weekday-W" />
          Wednesday
        </label>
        <label>
          <input type="checkbox" bind:checked={$state.weekdays.Th} name="weekday-Th" />
          Thursday
        </label>
        <label>
          <input type="checkbox" bind:checked={$state.weekdays.F} name="weekday-F" />
          Friday
        </label>
        <label>
          <input type="checkbox" bind:checked={$state.weekdays.Sa} name="weekday-Sa" />
          Saturday
        </label>
      </fieldset>
    {/if}

    <div>
      <a href={$replayUrl} title={`Feed URL: ${$replayUrl}`} class="subscribe-url">Subscribe</a>
    </div>
  </form>

  <ItemPreview {feed} {start} {state} />
{/if}

<style>
  h1 {
    font-size: 1.5rem;
  }
  form {
    display: block;
  }
  .subscribe-url {
    width: 100%;
  }
</style>
