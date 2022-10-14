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
    Have you ever come across an awesome new (to you!) podcast with a deep back catalog? A
    serialized crime drama or a deep dive into history?
  </p>

  <p>
    So you add it to your podcast player, and... load up all the episodes at once? Add only the
    first few? What about your other podcasts?
  </p>

  <p>
    What if you could rewind? Listen from the beginning as if the podcast was just starting out? You
    set the replay schedule. Once caught up, new episodes will continue appearing as they're
    published.
  </p>

  <p><a href="#enter-a-feed-url">Let's go listen to a podcast!</a></p>

  <div class="warnings">
    <h3>Warnings and Caveats</h3>
    <p>
      This is <em>brand new</em> software built for fun. While a good faith effort is made to preserve
      the integrity of the original feed, compatibilty cannot be guaranteed.
    </p>
    <p>
      Nothing is rehosted and caching is minimal, so there isn't much that can be done if the
      publisher deletes episodes or moves things around too much.
    </p>
  </div>
</main>

<Footer />

<style>
  main > * {
    max-width: 37rem;
    margin: 1em auto;
  }
  .warnings {
    margin-top: 5em;
    color: #666;
  }
</style>
