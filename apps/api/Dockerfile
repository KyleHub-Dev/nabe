FROM rust:1-bookworm AS build
WORKDIR /src/apps/api
COPY apps/api/Cargo.toml ./
COPY apps/api/Cargo.lock ./
COPY apps/api/src src
COPY apps/api/migrations migrations
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/apps/api/target/release/nabe-api /usr/local/bin/nabe-api
EXPOSE 8080
ENTRYPOINT ["nabe-api"]
