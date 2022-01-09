<script lang="ts">
  import Header from '../components/Header.svelte';
  import FeedForm from '../components/FeedForm.svelte';
  import ItemPreview from '../components/ItemPreview.svelte';
  import ConfigureForm from '../components/ConfigureForm.svelte';
  import { queryStore, replayUrlStore } from '../util/state';
  import { feedSummaryStore } from '../util/fetchFeedSummary';
  import SubscribeLinks from '../components/SubscribeLinks.svelte';

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

<Header>
  <FeedForm {state} />
</Header>

{#await $feed}
  Loading feed...
{:then feed}
  <ConfigureForm {feed} {state} />
  <SubscribeLinks url={$replayUrl} />
  <ItemPreview {feed} {start} {state} />
{:catch error}
  Error: {error}
{/await}

<style>
</style>
