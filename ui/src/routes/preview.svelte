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

    {#if $state.uri !== feed.uri}
      <div class="autodiscovery-warning">
        <h3>Does everything look OK?</h3>
        <p>
          It looks like you entered a podcast page rather than the actual feed URL. That's ok! I
          tried to find the feed, but sometimes that doesn't work out so well. For what it's worth,
          this is the link I found: <code>{feed.uri}</code>
        </p>
        <p>
          If this episode preview doesn't look quite right, maybe see if you can find the link
          yourself? Look for buttons with words like "Feed" or "RSS" or "Subscribe".
        </p>
      </div>
    {/if}

    <ItemPreview {feed} {state} />
  {:catch error}
    <div class="error">
      <img src="/bug.svg" width="50px" alt="hourglass icon" />
      {error}
    </div>
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
  .loading,
  .error {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    margin: 2em;
    gap: 0.5em;
  }
  .autodiscovery-warning {
    font-size: 0.9em;
    background: #ffe4b2;
    padding: 0.2em 1em 0.1em;
    border-radius: 0.2em;
  }
  noscript {
    display: block;
    margin: 1em;
    text-align: center;
  }
</style>
