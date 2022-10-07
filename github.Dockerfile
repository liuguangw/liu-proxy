# github action 优化构建
FROM debian:buster-slim
WORKDIR /home/liu-proxy
COPY ./liu-proxy /usr/local/bin/
COPY ./other_files .
EXPOSE 8001
CMD ["liu-proxy", "server"]