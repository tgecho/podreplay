<script lang="ts">
  import { fade } from 'svelte/transition';
  import type { Writable } from 'svelte/store';
  import type { State } from '../util/state';
  import type { FeedSummary } from '../util/fetchFeedSummary';

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

      <fieldset class="freq" class:plural={$state.interval > 1}>
        <label class:selected={$state.freq === 'm'}>
          <input type="radio" name="freq" bind:group={$state.freq} value="m" />
          Month
        </label>
        <label class:selected={$state.freq === 'w'}>
          <input type="radio" name="freq" bind:group={$state.freq} value="w" />
          Week
        </label>
        <label class:selected={$state.freq === 'd'}>
          <input type="radio" name="freq" bind:group={$state.freq} value="d" />
          Day
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
  h2 {
    text-align: center;
    font-size: 1.25rem;
    margin: 1.5em 0 0.5em;
  }
  h3 {
    font-size: 1rem;
  }
  label,
  input[type='radio'],
  input[type='checkbox'] {
    cursor: pointer;
  }
  .title {
    position: relative;
    margin: 0.75em 0;
    align-items: center;
    flex-wrap: wrap;
    justify-content: center;
    display: flex;
    gap: 0.5em;
  }
  .title input {
    font-size: 1.5em;
    border: none;
    border-bottom: 1px dashed var(--accent-fg-color);
    border-radius: 0.5em;
    padding: 0.35em;
    color: var(--main-fg-color);
    font-size: 1em;
    margin: 0;
    background: none;
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
    justify-content: center;
  }
  .radiosets {
    display: flex;
    gap: 0.5em;
    flex-wrap: wrap;
    flex: 1 20em;
    justify-content: center;
  }
  fieldset {
    border: none;
    margin: 0;
    padding: 0;
    border-bottom: 1px dashed var(--accent-fg-color);
    border-radius: 0.5em;
    display: flex;
    /* flex: 1 1 50%; */
  }
  fieldset label {
    padding: 0.25em 0.5em;
    display: flex;
    gap: 0.1em;
    align-items: baseline;
    flex-wrap: wrap;
    justify-content: center;
  }
  fieldset.freq label::after {
    content: 's';
    opacity: 0;
  }
  fieldset.freq.plural label::after {
    content: 's';
    opacity: 1;
  }
</style>
