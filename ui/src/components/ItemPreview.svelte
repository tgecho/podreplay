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
  <thead>
    <tr>
      <th class="first"><span>Choose a first episode <sup>(optional)</sup></span></th>
      <th class="title">Title</th>
      <th class="original">Original</th>
      <th class="rescheduled">Rescheduled</th>
      <th class="last"><span>Choose a last episode <sup>(optional)</sup></span></th>
    </tr>
  </thead>
  <tbody>
    {#each feed?.items as item, index (item.id)}
      <tr class:skipped={!rescheduled[index]}>
        <td class="constrain first">
          {#if item.timestamp == $state.first}
            <button type="button" on:click={() => ($state.first = null)}>▲</button>
          {:else if $state.last ? item.timestamp <= $state.last : true}
            <input type="radio" bind:group={$state.first} value={item.timestamp} id={'f' + index} />
            <label for={'f' + index}>Start Here</label>
          {/if}
        </td>
        <th class="title" title={item.title}>{item.title}</th>
        <td class="original">
          <time datetime={item.timestamp}>{format(new Date(item.timestamp), 'MMM do, y')}</time>
        </td>
        <td class="rescheduled">
          {#if rescheduled[index]}
            <time datetime={rescheduled[index].toISOString()}
              >{format(rescheduled[index], 'MMM do, y')}</time
            >
          {:else}
            Skip
          {/if}
        </td>
        <td class="constrain last">
          {#if item.timestamp == $state.last}
            <button type="button" on:click={() => ($state.last = null)}>▼</button>
          {:else if $state.first ? item.timestamp >= $state.first : true}
            <input type="radio" bind:group={$state.last} value={item.timestamp} id={'l' + index} />
            <label for={'l' + index}>End Here</label>
          {/if}
        </td>
      </tr>
    {/each}
  </tbody>
</table>

<style>
  table {
    margin: 1em;
    width: calc(100% - 2em);
    table-layout: fixed;
    border-spacing: 0;
  }
  th,
  td {
    padding: 0.2em 0.3em;
    white-space: nowrap;
  }
  thead {
    position: sticky;
    top: 0;
    z-index: 1;
  }
  thead th {
    background: var(--main-bg-color);
  }
  thead .first,
  thead .last {
    vertical-align: middle;
  }
  thead .first sup,
  thead .last sup {
    font-size: 0.7em;
    line-height: 1;
    font-style: italic;
    vertical-align: middle;
    font-weight: normal;
  }
  thead .first span {
    position: absolute;
    right: 100%;
    bottom: 0;
    transform-origin: bottom right;
    transform: rotate(270deg);
  }
  thead .last span {
    position: absolute;
    left: 100%;
    bottom: 0;
    transform-origin: bottom left;
    transform: rotate(90deg);
  }
  tbody .title,
  tbody .original,
  tbody .rescheduled {
    overflow: hidden;
    text-overflow: ellipsis;
    transition: opacity 0.75s;
  }
  th {
    text-align: left;
  }
  td {
    white-space: nowrap;
  }
  tr:nth-child(even) {
    background-color: #fff;
  }

  .skipped .title,
  .skipped .original,
  .skipped .rescheduled {
    opacity: 0.4;
    text-decoration: line-through;
  }

  thead .original,
  thead .rescheduled {
    width: 8em;
  }
  thead .first,
  thead .last {
    width: 2em;
  }

  .title {
    font-weight: 500;
  }
  thead .title {
    font-weight: bold;
  }
  .constrain {
    flex: 0 0 7em;
    white-space: nowrap;
    position: relative;
    text-align: center;
  }
  .constrain label {
    display: none;
  }
  .constrain button {
    padding: 0;
    border: 0 solid #000;
    font-size: 1em;
    background: none;
    line-height: 0.6;
  }
  .first button {
    border-top-width: 2px;
  }
  .last button {
    border-bottom-width: 2px;
  }
  .constrain input {
    margin: 0;
  }
  /* @media (max-width: 30em) {
    table {
      display: block;
    }
    thead {
      display: none;
    }
    tr {
      display: flex;
      flex-wrap: wrap;
    }
    .title {
      flex: 1 1 100%;
      font-weight: bold;
      text-align: left;
    }
    .original::before {
      content: 'Originally: ';
    }
    .rescheduled::before {
      content: 'Replay on: ';
    }
    .skipped .rescheduled::before {
      content: 'Skipped';
    }
    .constrain {
      display: flex;
    }
  } */
</style>
