[PodReplay](https://podreplay.com) is a service that allows replaying podcast feeds as if they just started.

Behind the scenes post: https://esimmler.com/slowly-listening-to-old-podcasts/

# Development

## Install

You'll need [Rust](https://rustup.rs/), [Node](https://nodejs.org/) and [pnpm](https://pnpm.io/) installed.

To create a local test/dev database file:

```sh
$ cargo install sqlx-cli --no-default-features --features sqlite,rustls
$ sqlx database create
```

Start the server:

```sh
$ cargo run
```

Build the WASM module (unfortunately I didn't get this fully automated/integrated, so you'll need to run the build command every time you update the underlying rust code):

```sh
$ cargo install wasm-pack # just the first time
$ wasm-pack build --target web lib_wasm --profile release
```

Start up the client in dev server mode:

```sh
$ pnpm install
$ pnpm dev
```

Open the client frontend at http://localhost:3000/ and dev away.

## Run server tests

```sh
$ cargo test
```

## Deploy to Fly.io

Obviously you'll need the appropriate credentials in place. I don't remember what preliminary steps I had to do on top of the base [flyctl install instructions](https://fly.io/docs/hands-on/install-flyctl/).

```sh
$ flyctl deploy
```
