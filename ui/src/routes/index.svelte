<script lang="ts">
  import Header from '../components/Header.svelte';
  import FeedForm from '../components/FeedForm.svelte';
  import Footer from '../components/Footer.svelte';
</script>

<svelte:head>
  <title>PodReplay</title>
</svelte:head>

<Header>
  <FeedForm />
</Header>

<main>
  <h2>Hello!</h2>
  <p>
    Have you ever come across an awesome new (to you!) podcast with a deep back catalog of episodes?
    Maybe a serialized crime drama or a deep dive into history?
  </p>

  <p>
    So you add it to your podcast player, and... load up all of the episodes at once? Maybe manually
    add only the first few? What about all of your other podcasts?
  </p>

  <p>
    What if instead of all that, you could listen from the beginning as if the podcast really was
    just starting out? You set the schedule, and once caught up you'll continue getting new episodes
    whenever they're published.
  </p>

  <p><strong>Let's go listen to a podcast!</strong></p>

  <h3 class="warnings">Warnings and Caveats</h3>
  <p>
    This is <em>brand new</em> software built for fun. While a good faith effort is made to preserve
    the integrity of the original podcast feeds, compatibilty cannot be guaranteed.
  </p>
  <p>
    Nothing is rehosted and caching is minimal, so if the podcast publisher deletes episodes or
    moves things around too much there isn't much that can be done.
  </p>
</main>

<Footer />

<style>
  main > * {
    max-width: 37rem;
    margin: 1em auto;
  }
  .warnings {
    margin-top: 5em;
  }
</style>
