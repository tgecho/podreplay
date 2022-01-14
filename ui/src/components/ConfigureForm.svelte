<script lang="ts">
  import { fade } from 'svelte/transition';
  import type { Writable } from 'svelte/store';
  import type { State } from '../util/state';
  import type { FeedSummary } from '../util/fetchFeedSummary';
  import { formatForInput, formatForUrl } from '../util/dates';

  export let feed: FeedSummary;
  export let state: Writable<State>;

  const defaultTitle = `${feed.title} (PodReplay)`;
  function onTitleFocus() {
    if (!$state.title) {
      $state.title = defaultTitle;
    }
  }
  function onTitleBlur() {
    if ($state.title === defaultTitle) {
      $state.title = '';
    }
  }

  $: startString = formatForInput($state.start);
  function updateStart(ev: { currentTarget: HTMLInputElement }) {
    $state.start = new Date(ev.currentTarget.value);
  }

  $: s = $state.interval > 1 ? 's' : '';
</script>

<form target="/preview" on:submit|preventDefault>
  <input type="hidden" name="uri" value={$state.uri} />

  <label class="title">
    <h3>Title</h3>
    <input
      name="title"
      bind:value={$state.title}
      placeholder={`${feed.title} (PodReplay)`}
      on:focus={onTitleFocus}
      on:blur={onTitleBlur}
    />
  </label>

  <label class="start">
    <h3>Starting</h3>
    <input type="datetime-local" value={startString} on:change={updateStart} />
  </label>

  <div class="schedule">
    <h3>Schedule an episode every</h3>
    <div class="radiosets">
      <fieldset class="interval">
        <label class:selected={$state.interval === 1}>
          <input type="radio" name="interval" bind:group={$state.interval} value={1} /> One
        </label>
        <label class:selected={$state.interval === 2}>
          <input type="radio" name="interval" bind:group={$state.interval} value={2} /> Two
        </label>
        <label class:selected={$state.interval === 3}>
          <input type="radio" name="interval" bind:group={$state.interval} value={3} /> Three
        </label>
      </fieldset>

      <fieldset class="freq">
        <label class:selected={$state.freq === 'm'}>
          <input type="radio" name="freq" bind:group={$state.freq} value="m" />
          Month{s}
        </label>
        <label class:selected={$state.freq === 'w'}>
          <input type="radio" name="freq" bind:group={$state.freq} value="w" />
          Week{s}
        </label>
        <label class:selected={$state.freq === 'd'}>
          <input type="radio" name="freq" bind:group={$state.freq} value="d" />
          Day{s}
        </label>
      </fieldset>

      {#if $state.freq === 'w'}
        <fieldset class="weekdays" transition:fade>
          <label>
            <input type="checkbox" bind:checked={$state.weekdays.Su} name="weekday-Su" />
            Sun
          </label>
          <label>
            <input type="checkbox" bind:checked={$state.weekdays.M} name="weekday-M" />
            Mon
          </label>
          <label>
            <input type="checkbox" bind:checked={$state.weekdays.Tu} name="weekday-Tu" />
            Tue
          </label>
          <label>
            <input type="checkbox" bind:checked={$state.weekdays.W} name="weekday-W" />
            Wed
          </label>
          <label>
            <input type="checkbox" bind:checked={$state.weekdays.Th} name="weekday-Th" />
            Thu
          </label>
          <label>
            <input type="checkbox" bind:checked={$state.weekdays.F} name="weekday-F" />
            Fri
          </label>
          <label>
            <input type="checkbox" bind:checked={$state.weekdays.Sa} name="weekday-Sa" />
            Sat
          </label>
        </fieldset>
      {/if}
    </div>
  </div>
</form>

<style>
  form {
    flex: 1 1 75%;
  }
  h3 {
    font-size: 1rem;
    margin: 0.6em 0 0.5em;
  }
  input {
    font-size: 1em;
    margin: 0;
  }
  input[type='radio'],
  input[type='checkbox'] {
    margin: 0.2em 0.25em;
  }
  .title,
  .start {
    position: relative;
    margin: 0.75em 0;
    align-items: center;
    flex-wrap: wrap;
    justify-content: flex-start;
    display: flex;
    gap: 0.5em;
  }
  .title input {
    flex: 1 1 20em;
  }
  .title input::placeholder {
    color: var(--accent-fg-color);
  }

  .schedule {
    display: flex;
    gap: 0.5em;
    flex-wrap: wrap;
    align-items: center;
    justify-content: flex-start;
  }
  .radiosets {
    display: flex;
    gap: 0.5em;
    flex-wrap: wrap;
    flex: 1 20em;
    justify-content: flex-start;
  }
  fieldset {
    margin: 0;
    padding: 0;
    border: 1px dotted var(--accent-fg-color);
    border-radius: 0.2em;
    background: #fff;
    display: flex;
  }
  fieldset label {
    padding: 0.25em 0.5em;
    display: flex;
    gap: 0.1em;
    align-items: baseline;
    flex-wrap: wrap;
    justify-content: center;
  }
</style>
