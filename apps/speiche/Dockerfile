FROM golang:1.22-bookworm AS build
WORKDIR /src
COPY apps/speiche/go.mod ./
COPY apps/speiche ./
RUN go build -o /out/speiche .

FROM debian:bookworm-slim
COPY --from=build /out/speiche /usr/local/bin/speiche
USER 65532:65532
ENTRYPOINT ["speiche"]
