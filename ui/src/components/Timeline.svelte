<script lang="ts">
  // import Timeline from '../components/Timeline.svelte';
  // import { parseJSON } from 'date-fns';

  // const replayStartTimestamp = new Date();
  // const replayGoal = 'shift';
  // const timestamps = [
  //   { label: 'One', timestamp: parseJSON('2021-03-15T05:20:10.123Z') },
  //   { label: 'Two', timestamp: parseJSON('2000-03-25T05:20:10.123Z') },
  // ];

  export let replayStartTimestamp = new Date();
  export let originalAnchorTimestamp = new Date();
  export let replayGoal: 'shift' | 'catchup';
  export let timestamps: { label: string; timestamp: Date }[];

  import { add, differenceInDays, eachDayOfInterval } from 'date-fns';

  let firstTimestamp = timestamps[0]?.timestamp;
  let lastTimestamp = timestamps[timestamps.length - 1]?.timestamp;
  let endTimestamp = add(lastTimestamp || firstTimestamp, { days: 2 });
  $: console.log({ start: firstTimestamp, end: endTimestamp });
  $: days = eachDayOfInterval({ start: firstTimestamp, end: endTimestamp });
</script>

<!-- <Timeline {replayStartTimestamp} {replayGoal} {timestamps} /> -->

<div class="wrapper">
  {#each days as day, index}
    <div class="tick" style={`left: ${index * 10}px`}>.</div>
  {/each}
</div>

<style>
  .wrapper {
    position: relative;
  }
  .tick {
    border-left: 1px solid #000;
    height: 10px;
    position: absolute;
  }
</style>
