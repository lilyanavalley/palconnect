
# Build steps
FROM rust:1.89 AS builder

ENV RUST_BACKTRACE=full

RUN apt-get update -y \
&& apt-get install -y --no-install-recommends build-essential clang pkg-config libssl-dev ca-certificates openssl
WORKDIR /usr/src/palconnect
COPY . .

RUN cargo install --path .

# Run steps
FROM debian:bullseye-slim

RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/palconnect /usr/local/bin/palconnect

ENV RUST_BACKTRACE=1
ENV RUST_LOG="warn"
CMD ["palconnect"]
