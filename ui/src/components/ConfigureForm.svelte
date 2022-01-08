<script lang="ts">
  import type { Writable } from 'svelte/store';
  import type { State } from '../util/state';
  import type { FeedSummary } from '../util/fetchFeedSummary';

  export let feed: FeedSummary;
  export let state: Writable<State>;

  $: s = $state.interval > 1 ? 's' : '';
</script>

<form target="/preview" on:submit|preventDefault>
  <input type="hidden" name="uri" value={$state.uri} />

  <fieldset>
    Custom Title:
    <input name="title" bind:value={$state.title} placeholder={`${feed.title} (PodReplay)`} />
  </fieldset>

  <fieldset>
    A replayed episode every
    <input type="number" name="interval" bind:value={$state.interval} min={1} max={10} />
  </fieldset>

  <fieldset>
    <label>
      <input type="radio" name="freq" bind:group={$state.freq} value="m" />
      Month{s}
    </label>
    <label>
      <input type="radio" name="freq" bind:group={$state.freq} value="w" />
      Week{s}
    </label>
    <label>
      <input type="radio" name="freq" bind:group={$state.freq} value="d" /> Day{s}
    </label>
  </fieldset>
  {#if $state.freq === 'w'}
    <fieldset>
      On
      <label>
        <input type="checkbox" bind:checked={$state.weekdays.Su} name="weekday-Su" />
        Sunday
      </label>
      <label>
        <input type="checkbox" bind:checked={$state.weekdays.M} name="weekday-M" />
        Monday
      </label>
      <label>
        <input type="checkbox" bind:checked={$state.weekdays.Tu} name="weekday-Tu" />
        Tuesday
      </label>
      <label>
        <input type="checkbox" bind:checked={$state.weekdays.W} name="weekday-W" />
        Wednesday
      </label>
      <label>
        <input type="checkbox" bind:checked={$state.weekdays.Th} name="weekday-Th" />
        Thursday
      </label>
      <label>
        <input type="checkbox" bind:checked={$state.weekdays.F} name="weekday-F" />
        Friday
      </label>
      <label>
        <input type="checkbox" bind:checked={$state.weekdays.Sa} name="weekday-Sa" />
        Saturday
      </label>
    </fieldset>
  {/if}
</form>
