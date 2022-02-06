Amazon Podcasts -- nope
Spotify -- nope
Stitcher -- nope
SoundCloud -- nope (example: https://soundcloud.com/esc-podcast)
Gimlet -- nope (example: https://gimletmedia.com/shows/the-habitat)

https://www.dancarlin.com/hardcore-history-series/ has a meta tag with (apparently) a single blog entry. Not sure if we need a special rule or just more aggressive/intelligent autodiscovery code.

Throws an error in the firefox preview (though the feed appears to work due to lenient clients)
I guess I need to add the itunes namespace?
https://podreplay.com/replay?start=2022-01-30T04:32:29Z&rule=1d&first=2021-01-01T05:00:00Z&uri=https%3A%2F%2Fstrangelandpodcast.com%2Ffeed%2F

# https://www.podchaser.com/podcasts/fat-mascara-90926

Has this blob of ld+json with the feed url in it... might be something not too specific to them?

<script data-rh="true" type="application/ld+json">{"@context":"http://schema.org","@type":"PodcastSeries","@id":"https://www.podchaser.com/podcasts/fat-mascara-90926","accessMode":"auditory","genre":"Arts","description":"Beauty journalists (and friends!) Jessica Matlin and Jennifer Sullivan turn up the volume and bring you the big, juicy, world of beauty twice a week. On Tuesday episodes, they share their insider access to the beauty industry, candid stories of their beauty adventures, and the best perfumes, skinca…","identifier":90926,"image":"https://assets.pippa.io/shows/619566352eacc3a360702519/show-cover.jpg","name":"Fat Mascara","url":"https://www.podchaser.com/podcasts/fat-mascara-90926","webFeed":"https://access.acast.com/rss/e6a92aaf-8518-4cbc-a4be-2f760ded435a/","creator":[{"@type":"Person","@id":"https://www.podchaser.com/creators/jennifer-g-sullivan-107tLwoxda","name":"Jennifer G. Sullivan"},{"@type":"Person","@id":"https://www.podchaser.com/creators/jessica-matlin-107ZzonI2z","name":"Jessica Matlin"},{"@type":"Person","@id":"https://www.podchaser.com/creators/undefined"}],"aggregateRating":{"@type":"AggregateRating","ratingValue":5,"ratingCount":1},"startDate":"2016-02-23 20:00:00"}</script>

# https://www.wnycstudios.org/podcasts/dolly-partons-america

We're only finding an empty feed they've mistakenly linked from a metatag. There don't seem to be any other recognizable links.
