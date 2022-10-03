FROM rust:latest as builder
WORKDIR /usr/src/liu-proxy
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
WORKDIR /home/liu-proxy
RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/liu-proxy /usr/local/bin/
COPY --from=builder /usr/src/liu-proxy/config /home/liu-proxy/
EXPOSE 7008
CMD ["liu-proxy"]