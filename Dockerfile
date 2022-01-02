# Build Server and WASM
FROM rust:1.57.0 AS server_and_wasm

WORKDIR /usr/src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/target \
    cargo install sqlx-cli

COPY Cargo.toml Cargo.lock ./
COPY lib ./lib
COPY lib_wasm ./lib_wasm
COPY server ./server
COPY migrations ./migrations

ENV DATABASE_URL="sqlite:///usr/src/db.sqlite"
RUN sqlx database create && sqlx migrate run

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/target \
    cargo install --path ./server

RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/target \
    cargo install --path ./server

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

# Build final container
FROM ubuntu AS app
RUN apt update -y && apt install openssl -y
COPY --from=server_and_wasm /usr/local/cargo/bin/podreplay ./app/
# TODO: manage this with litestream and a boot time config?
COPY --from=server_and_wasm /usr/src/db.sqlite ./test.sqlite
COPY --from=frontend /usr/src/ui/build ./ui/build
CMD "./app/podreplay"
