<script lang="ts">
  import Header from '../components/Header.svelte';
  import FeedForm from '../components/FeedForm.svelte';
  import ItemPreview from '../components/ItemPreview.svelte';
  import ConfigureForm from '../components/ConfigureForm.svelte';
  import { queryStore } from '../util/state';
  import { feedSummaryStore } from '../util/fetchFeedSummary';
  import SubscribeLinks from '../components/SubscribeLinks.svelte';
  import Footer from '../components/Footer.svelte';
  import { browser } from '$app/env';

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

<noscript>
  Javascript is required for the full preview experience, but you can still generate a replay feed
  with the form above!
</noscript>

{#if browser}
  {#await $feed}
    <div class="loading">
      <img src="/hourglass.svg" width="50px" alt="hourglass icon" />
      &nbsp;&nbsp;Loading Feed...
    </div>
  {:then feed}
    <div class="top">
      <ConfigureForm {feed} {state} />
      <SubscribeLinks {state} feedUri={feed.uri} />
    </div>
    <ItemPreview {feed} {state} />
  {:catch error}
    Error: {error}
  {/await}
{/if}

<Footer />

<style>
  .top {
    display: flex;
    flex-wrap: wrap;
    justify-content: stretch;
    align-items: flex-start;
    gap: 1em;
    margin: 1.5em 0;
  }
  .loading {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    margin: 2em;
    gap: 0.5em;
  }
  noscript {
    display: block;
    margin: 1em;
    text-align: center;
  }
</style>
