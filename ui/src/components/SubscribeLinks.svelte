<script lang="ts">
  export let url: string;
</script>

<div>
  <a href={url} target="replay" title={`Feed URL: ${url}`} class="subscribe-url">Subscribe</a>
</div>
