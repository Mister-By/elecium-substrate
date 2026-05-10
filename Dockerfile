FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY target/release/solochain-template-node /usr/local/bin/node

RUN useradd -m -u 1000 substrate
USER substrate

EXPOSE 30333 9944

ENTRYPOINT ["/usr/local/bin/node"]
