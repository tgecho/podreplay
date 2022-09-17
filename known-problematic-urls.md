TODO: disable warning for directly entered itunes links

Amazon Podcasts -- nope
Spotify -- nope
Stitcher -- nope
SoundCloud -- nope (example: https://soundcloud.com/esc-podcast)
Gimlet -- nope (example: https://gimletmedia.com/shows/the-habitat)

# It seems when we skip episodes we may be leaving a bunch of whitespace in the output file... need to clean this up.

https://www.dancarlin.com/hardcore-history-series/ has a meta tag with (apparently) a single blog entry. Not sure if we need a special rule or just more aggressive/intelligent autodiscovery code.

# https://www.alieward.com/ologies

We autodiscover a weird truncated (by squarespace) feed that _doesn't have podcast enclosures!_ (https://www.alieward.com/ologies?format=rss)
I can't see anything on the page to grab onto
We can find the apple podcast with https://itunes.apple.com/search?media=podcast&entity=podcast&limit=2&term=ologies
... but I'm not sure how to reliably construct the search term as it doesn't seem very flexible

Maybe this is a job for the search api at listennotes?

- https://www.listennotes.com/search/?q=Ologies%20Episodes%20alie%20ward&sort_by_date=0&scope=episode&offset=0&language=Any%20language&len_min=0
- https://www.listennotes.com/search/?q=https%3A%2F%2Fwww.alieward.com%2Fologies&scope=episode

https://www.wnycstudios.org/podcasts/radiolab/podcasts has a meta tag to an empty feed. All of the actual feed links are locked behind a javascript popup

Throws an error in the firefox preview (though the feed appears to work due to lenient clients)
I guess I need to add the itunes namespace?
https://podreplay.com/replay?start=2022-01-30T04:32:29Z&rule=1d&first=2021-01-01T05:00:00Z&uri=https%3A%2F%2Fstrangelandpodcast.com%2Ffeed%2F

# https://www.podchaser.com/podcasts/fat-mascara-90926

Has this blob of ld+json with the feed url in it... might be something not too specific to them?

<script data-rh="true" type="application/ld+json">{"@context":"http://schema.org","@type":"PodcastSeries","@id":"https://www.podchaser.com/podcasts/fat-mascara-90926","accessMode":"auditory","genre":"Arts","description":"Beauty journalists (and friends!) Jessica Matlin and Jennifer Sullivan turn up the volume and bring you the big, juicy, world of beauty twice a week. On Tuesday episodes, they share their insider access to the beauty industry, candid stories of their beauty adventures, and the best perfumes, skinca…","identifier":90926,"image":"https://assets.pippa.io/shows/619566352eacc3a360702519/show-cover.jpg","name":"Fat Mascara","url":"https://www.podchaser.com/podcasts/fat-mascara-90926","webFeed":"https://access.acast.com/rss/e6a92aaf-8518-4cbc-a4be-2f760ded435a/","creator":[{"@type":"Person","@id":"https://www.podchaser.com/creators/jennifer-g-sullivan-107tLwoxda","name":"Jennifer G. Sullivan"},{"@type":"Person","@id":"https://www.podchaser.com/creators/jessica-matlin-107ZzonI2z","name":"Jessica Matlin"},{"@type":"Person","@id":"https://www.podchaser.com/creators/undefined"}],"aggregateRating":{"@type":"AggregateRating","ratingValue":5,"ratingCount":1},"startDate":"2016-02-23 20:00:00"}</script>

# https://www.wnycstudios.org/podcasts/dolly-partons-america

We're only finding an empty feed they've mistakenly linked from a metatag. There don't seem to be any other recognizable links.

# https://hidden-brain.simplecast.com/

This is a SPA page so there are no links. Probably need to be found with a search.

# https://feed.thisamericanlife.org/talpodcast

https gets a tls handshake eof from the feed link found in a metatag
we might be able to detect this sort of thing and attempt http

# https://www.youtube.com/mythicalkitchen

video links? I guess I'll we'd need to do is relax the type check

# https://themoth.org

The actual feed links are just one hop away... should we follow "subscribe" or "listen" links at least one hop? same domain limit?

# Links to test

- https://thememorypalace.us/
- https://atp.fm/
- https://podcasts.google.com/feed/aHR0cHM6Ly9mZWVkcy5tZWdhcGhvbmUuZm0vc3R1ZmZ5b3VzaG91bGRrbm93
- https://www.dungeonsanddaddies.com/
- https://gimletmedia.com/shows/reply-all
