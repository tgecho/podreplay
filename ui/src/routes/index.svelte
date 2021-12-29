<script context="module" lang="ts">
  import { FeedSummary, fetchFeedSummary } from '../util/fetchFeedSummary';
  import type { Load } from '@sveltejs/kit';

  export const load: Load = async ({ page, fetch }) => {
    const uri = page.query.get('uri');
    const feed = uri ? await fetchFeedSummary(uri, fetch) : null;
    return { props: { feed } };
  };
</script>

<script lang="ts">
  import { format, parseISO } from 'date-fns';
  import { debounce } from 'lodash';
  import FeedForm from '../components/FeedForm.svelte';
  import { page } from '$app/stores';
  import { browser } from '$app/env';
  import { reschedule } from '../util/reschedule';
  export let feed: FeedSummary | null = null;

  $: rescheduled = feed ? reschedule(feed.items) : [];

  // The feed URI
  const uri = $page.query.get('uri') || '';

  const updateUrl = debounce((name: string, value: string | number) => {
    if (value) {
      $page.query.set(name, value.toString());
    }
    if (browser) {
      history.replaceState({}, '', `?${$page.query.toString()}`);
    }
  }, 250);
</script>

<h1>PodReplay</h1>

<FeedForm {uri} />

{#if feed}
  <b>Feed Title:</b>
  {feed.title}

  <!-- <form target="">
    I want to start with
    <input type="hidden" name="uri" value={uri} />
    <Select options={firstOptions} name="first" bind:value={first} />
    <input type="hidden" name="first" bind:value={first} />
    <input type="text" bind:value={firstText} list="episodes" />
    <datalist id="episodes">
      {#each firstOptions as episode}
        <option value={episode.label} />
      {/each}
    </datalist>
    <br />and
    <input type="range" min={0.1} max={10} step={0.1} name="rate" bind:value={rate} />
    <br />This will be about {'{TODO}'} episodes per week.
    <button>Save</button>
  </form> -->

  <table class="timeline">
    {#await rescheduled then rescheduled}
      {#each feed?.items as item, index}
        <tr>
          <th>{item.title}</th>
          <!-- <td>{item.timestamp}</td> -->
          <td>{format(new Date(item.timestamp), 'MMM do, y')}</td>
          {#if rescheduled[index]}<td>{format(rescheduled[index], 'MMM do, y')}</td>{/if}
        </tr>
      {/each}
    {/await}
  </table>
{/if}

<style>
  h1 {
    font-size: 1.5rem;
  }
  form {
    display: block;
  }
</style>
