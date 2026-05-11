FROM rust:1-bookworm AS build
WORKDIR /src
COPY apps/api/Cargo.toml apps/api/Cargo.toml
COPY apps/api/Cargo.lock apps/api/Cargo.lock
COPY apps/api/src apps/api/src
COPY apps/api/migrations apps/api/migrations
COPY packages/adguard-adapter packages/adguard-adapter
RUN cargo build --release --manifest-path apps/api/Cargo.toml

FROM debian:bookworm-slim
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/apps/api/target/release/nabe-api /usr/local/bin/nabe-api
EXPOSE 8080
ENTRYPOINT ["nabe-api"]
