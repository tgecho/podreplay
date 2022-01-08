<script lang="ts">
  import { format } from 'date-fns';
  import type { Writable } from 'svelte/store';
  import type { FeedSummary } from '../util/fetchFeedSummary';
  import type { State } from '../util/state';
  import { init, reschedule } from '../util/reschedule';

  let ready = false;
  init().then(() => (ready = true));

  export let feed: FeedSummary;
  export let start: string;
  export let state: Writable<State>;

  $: rescheduled = ready ? reschedule(feed, start, $state) : [];
</script>

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
        <label>
          <input
            type="radio"
            bind:group={$state.first}
            value={item.timestamp}
            disabled={$state.last ? item.timestamp >= $state.last : false}
          />
          first
        </label>
        {#if item.timestamp == $state.first}<button
            type="button"
            on:click={() => ($state.first = null)}>Clear</button
          >{/if}
      </td>
      <td>
        <label
          ><input
            type="radio"
            bind:group={$state.last}
            value={item.timestamp}
            disabled={$state.first ? item.timestamp <= $state.first : false}
          /> last</label
        >
        {#if item.timestamp == $state.last}<button
            type="button"
            on:click={() => ($state.last = null)}>Clear</button
          >{/if}
      </td>
    </tr>
  {/each}
</table>
