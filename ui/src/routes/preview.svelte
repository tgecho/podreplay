<script lang="ts">
  import Header from '../components/Header.svelte';
  import FeedForm from '../components/FeedForm.svelte';
  import ItemPreview from '../components/ItemPreview.svelte';
  import ConfigureForm from '../components/ConfigureForm.svelte';
  import { queryStore } from '../util/state';
  import { feedSummaryStore } from '../util/fetchFeedSummary';
  import SubscribeLinks from '../components/SubscribeLinks.svelte';

  const state = queryStore();
  const feed = feedSummaryStore(state);
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
  <div class="top">
    <ConfigureForm {feed} {state} />
    <SubscribeLinks {state} />
  </div>
  <ItemPreview {feed} {state} />
{:catch error}
  Error: {error}
{/await}

<style>
  .top {
    display: flex;
    flex-wrap: wrap;
    justify-content: stretch;
  }
</style>
