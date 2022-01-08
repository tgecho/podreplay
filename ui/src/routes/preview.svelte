<script lang="ts">
  import FeedForm from '../components/FeedForm.svelte';
  import ItemPreview from '../components/ItemPreview.svelte';
  import ConfigureForm from '../components/ConfigureForm.svelte';
  import { queryStore, replayUrlStore } from '../util/state';
  import { feedSummaryStore, fetchFeedSummary } from '../util/fetchFeedSummary';

  const state = queryStore();
  const feed = feedSummaryStore(state);
  const start = new Date().toISOString();
  const replayUrl = replayUrlStore(state, start);
</script>

<svelte:head>
  {#await $feed}
    <title>PodReplay: Loading feed...</title>
  {:then feed}
    <title>PodReplay: {feed.title}</title>
  {:catch}
    <title>PodReplay: Error</title>
  {/await}
</svelte:head>

<h1>PodReplay</h1>

<FeedForm {state} />

{#await $feed}
  Loading feed...
{:then feed}
  <br /><b>Title:</b>
  {feed.title}
  <br /><b>URI:</b>
  {feed.uri}

  <ConfigureForm {feed} {state} />

  <div>
    <a href={$replayUrl} title={`Feed URL: ${$replayUrl}`} class="subscribe-url">Subscribe</a>
  </div>

  <ItemPreview {feed} {start} {state} />
{:catch error}
  Error: {error}
{/await}

<style>
  h1 {
    font-size: 1.5rem;
  }
</style>
