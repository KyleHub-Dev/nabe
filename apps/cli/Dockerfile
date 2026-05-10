FROM golang:1.22-bookworm AS build
WORKDIR /src
COPY apps/cli/go.mod ./
COPY apps/cli ./
RUN go build -o /out/nabe .

FROM debian:bookworm-slim
COPY --from=build /out/nabe /usr/local/bin/nabe
USER 65532:65532
ENTRYPOINT ["nabe"]
