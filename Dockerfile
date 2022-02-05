#######################
# Build Server and WASM
FROM rust:1.58 AS server_and_wasm

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

RUN npm install -g pnpm \
    && npm config set store-dir /usr/src/ui/node_modules/.pnpm-store
COPY --from=server_and_wasm /usr/src/lib_wasm/pkg ./lib_wasm/pkg
COPY pnpm-*.yaml ./
COPY ui ./ui
WORKDIR /usr/src/ui
RUN --mount=type=cache,target=/usr/src/ui/node_modules \
    pnpm install
RUN --mount=type=cache,target=/usr/src/ui/node_modules \
    pnpm build

######################
# Build final container
FROM ubuntu:latest AS app

RUN apt update -y && apt install ca-certificates -y && apt clean -y

WORKDIR /app

COPY --from=litestream/litestream:0.3.7 /usr/local/bin/litestream /app/litestream

# Copy server binary
COPY --from=server_and_wasm /usr/local/cargo/bin/podreplay ./

# Copy the transpiled frontend
COPY --from=frontend /usr/src/ui/build ./ui

COPY litestream.yml /etc/

CMD ./litestream restore -v -if-db-not-exists db.sqlite && \
    ./litestream replicate -exec "./podreplay"
