FROM rust:latest as builder
WORKDIR /usr/src/liu-proxy
COPY . .
RUN cargo install --path . \
    && mkdir -p ./other_files \
    && cp -r ./config ./web ./other_files/

FROM debian:buster-slim
WORKDIR /home/liu-proxy
COPY --from=builder /usr/local/cargo/bin/liu-proxy /usr/local/bin/
COPY --from=builder /usr/src/liu-proxy/other_files /home/liu-proxy/
EXPOSE 8001
CMD ["liu-proxy", "server"]