#######################
# Build Server and WASM
FROM rust:1.57 AS server_and_wasm

WORKDIR /usr/src

# Install a few base tools first so they're cached
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/target \
    cargo install sqlx-cli
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build a sqlite DB so the sqlx macros have something to look at
ENV DATABASE_URL="sqlite:///usr/src/db.sqlite"
COPY migrations ./migrations
RUN sqlx database create && sqlx migrate run

# Copy all of our source files
COPY Cargo.toml Cargo.lock ./
COPY lib ./lib
COPY lib_wasm ./lib_wasm
COPY server ./server

# Build the server/wasm targets
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/target \
    cargo install --locked --path ./server
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/target \
    wasm-pack build --target web lib_wasm --profile release

#######################
# Build Svelte frontend
FROM node:17 AS frontend

WORKDIR /usr/src

RUN npm install -g pnpm
COPY --from=server_and_wasm /usr/src/lib_wasm/pkg ./lib_wasm/pkg
COPY ui ./ui
WORKDIR /usr/src/ui
RUN --mount=type=cache,target=/usr/src/node_modules \
    pnpm install
RUN pnpm build

######################
# Build final container
FROM ubuntu:latest AS app

# Copy server binary
COPY --from=server_and_wasm /usr/local/cargo/bin/podreplay /app/

# Copy the (empty) prebuilt database
# TODO: manage this with litestream and a boot time config?
COPY --from=server_and_wasm /usr/src/db.sqlite /test.sqlite

# Copy the transpiled frontend
COPY --from=frontend /usr/src/ui/build /ui/build

CMD "/app/podreplay"
