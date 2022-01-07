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
  import { format } from 'date-fns';
  import { debounce } from 'lodash-es';
  import FeedForm from '../components/FeedForm.svelte';
  import { page } from '$app/stores';
  import { browser } from '$app/env';
  import { reschedule } from '../util/reschedule';
  import { parseRule, ruleToString } from '../util/parseRule';
  import { sortedQueryString } from '../util/queryString';

  export let feed: FeedSummary | null = null;

  let rescheduled: (Date | null)[] = [];
  let replayFeedUrl = '';

  const start = new Date().toISOString();
  const uri = $page.url.searchParams.get('uri') || '';

  let first: string | undefined = $page.url.searchParams.get('first') || undefined;
  let last: string | undefined = $page.url.searchParams.get('last') || undefined;

  let ruleString = uri && $page.url.searchParams.get('rule');
  const rule = (ruleString && parseRule(ruleString)) || {};

  const updateQueryString = debounce(() => {
    const queryString = sortedQueryString($page.url.searchParams);

    // TODO: tighten up the formatted date to not include sub second precision
    replayFeedUrl = `${location.origin}/replay?start=${start}&${queryString}`;
    if (browser && queryString) history.replaceState({}, '', `?${queryString}`);
  }, 150);

  function updateQueryParam(name: string, value?: string | number) {
    if (value) {
      $page.url.searchParams.set(name, value.toString());
    } else {
      $page.url.searchParams.delete(name);
    }
    updateQueryString();
  }

  $: updateQueryParam('uri', uri);
  $: updateQueryParam('first', first);
  $: updateQueryParam('last', last);
  $: {
    if (rule.uri) {
      ruleString = ruleToString(rule);
      updateQueryParam('rule', ruleString);
    }
  }
  $: {
    if (feed && ruleString) {
      reschedule(feed.items, ruleString, start, first, last).then((r) => {
        rescheduled = r;
      });
    }
  }
</script>

<h1>PodReplay</h1>

<FeedForm {uri} />

{#if feed}
  <br /><b>Title:</b>
  {feed.title}
  <br /><b>URI:</b>
  {feed.uri}

  <form target="">
    <input type="hidden" name="uri" value={uri} />

    <fieldset>
      A replayed episode every
      <input type="number" name="interval" bind:value={rule.interval} min={1} max={10} />
    </fieldset>
    <fieldset>
      <label
        ><input type="radio" name="freq" bind:group={rule.freq} value="m" /> Month{rule.interval > 1
          ? 's'
          : ''}</label
      >
      <label
        ><input type="radio" name="freq" bind:group={rule.freq} value="w" /> Week{rule.interval > 1
          ? 's'
          : ''}</label
      >
      <label
        ><input type="radio" name="freq" bind:group={rule.freq} value="d" /> Day{rule.interval > 1
          ? 's'
          : ''}</label
      >
    </fieldset>
    {#if rule.freq === 'w'}
      <fieldset>
        On
        <label
          ><input type="checkbox" bind:checked={rule.weekdays.Su} name="weekday-Su" /> Sunday</label
        >
        <label
          ><input type="checkbox" bind:checked={rule.weekdays.M} name="weekday-M" /> Monday</label
        >
        <label
          ><input type="checkbox" bind:checked={rule.weekdays.Tu} name="weekday-Tu" /> Tuesday</label
        >
        <label
          ><input type="checkbox" bind:checked={rule.weekdays.W} name="weekday-W" /> Wednesday</label
        >
        <label
          ><input type="checkbox" bind:checked={rule.weekdays.Th} name="weekday-Th" /> Thursday</label
        >
        <label
          ><input type="checkbox" bind:checked={rule.weekdays.F} name="weekday-F" /> Friday</label
        >
        <label
          ><input type="checkbox" bind:checked={rule.weekdays.Sa} name="weekday-Sa" /> Saturday</label
        >
      </fieldset>
    {/if}

    <div>
      Subscribe
      <input readonly value={replayFeedUrl} class="subscribe-url" />
    </div>
  </form>

  <table class="timeline">
    <tr>
      <th>Title</th>
      <th>Original</th>
      <th>Shifted</th>
      <th colspan="2">Limit</th>
    </tr>
    {#each feed?.items as item, index (item.id)}
      <tr>
        <td>{item.title}</td>
        <td>{format(new Date(item.timestamp), 'MMM do, y')}</td>
        <td>
          {#if rescheduled[index]}
            {format(rescheduled[index], 'MMM do, y')}
          {:else}
            Skip
          {/if}
        </td>
        <td>
          <label
            ><input
              type="radio"
              bind:group={first}
              value={item.timestamp}
              disabled={last ? item.timestamp >= last : false}
            /> first</label
          >
          {#if item.timestamp == first}<button type="button" on:click={() => (first = undefined)}
              >Clear</button
            >{/if}
        </td>
        <td>
          <label
            ><input
              type="radio"
              bind:group={last}
              value={item.timestamp}
              disabled={first ? item.timestamp <= first : false}
            /> last</label
          >
          {#if item.timestamp == last}<button type="button" on:click={() => (last = undefined)}
              >Clear</button
            >{/if}
        </td>
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
  .subscribe-url {
    width: 100%;
  }
</style>
