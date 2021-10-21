<script context="module" lang="ts">
  import { FeedSummary, fetchFeedSummary } from '../util/fetchFeedSummary';

  export async function load({ page, fetch }) {
    const uri = page.query.get('uri');
    const feed = uri ? await fetchFeedSummary(uri, fetch) : null;
    return { props: { feed } };
  }
</script>

<script lang="ts">
  import { format } from 'date-fns';
  import { debounce } from 'lodash';
  import FeedForm from '../components/FeedForm.svelte';
  import { page } from '$app/stores';
  import { browser } from '$app/env';
  export let feed: FeedSummary | null = null;

  // The feed URI
  const uri = $page.query.get('uri');

  // The first episode timestamp to include
  let first = parseInt($page.query.get('first')) || feed?.items[0]?.timestamp;
  let firstOptions = feed?.items.map((i) => ({
    label: `${i.title} (originally ${format(i.timestamp * 1000, 'MMM do, y')})`,
    value: i.timestamp,
  }));
  let firstText = firstOptions.find((o) => o.value === first)?.label ?? '';
  $: first = firstOptions.find((o) => o.label === firstText)?.value ?? feed?.items[0]?.timestamp;

  // The date to start the feed
  let start = Math.round(Date.now() / 1000);

  let rate = parseInt($page.query.get('rate')) || 1;

  const updateUrl = debounce((name: string, value: unknown) => {
    if (value) {
      $page.query.set(name, value.toString());
    }
    if (browser) {
      history.replaceState({}, '', `?${$page.query.toString()}`);
    }
  }, 250);
  $: updateUrl('first', first);
  $: updateUrl('rate', rate);

  $: baseOffset = start - feed?.items.find((i) => i.timestamp >= first).timestamp;
  $: adjustedItems = feed?.items
    .filter((i) => i.timestamp >= first)
    .map((item) => {
      let offset = (item.timestamp - first) * rate + baseOffset;
      return { ...item, adjusted: item.timestamp + offset };
    });
</script>

<h1>PodReplay</h1>

<FeedForm {uri} />

{#if feed}
  <b>Feed Title:</b>
  {feed.title}

  {first}
  {rate}

  <form target="">
    I want to start with
    <input type="hidden" name="uri" value={uri} />
    <!-- <Select options={firstOptions} name="first" bind:value={first} /> -->
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
  </form>

  <table class="timeline">
    {#each adjustedItems as item}
      <tr>
        <th>{item.title}</th>
        <td>{format(item.timestamp * 1000, 'MMM do, y')}</td>
        <td>{format(item.adjusted * 1000, 'MMM do, y')}</td>
      </tr>
    {/each}
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
