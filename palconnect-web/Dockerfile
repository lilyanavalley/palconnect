# Build stage
FROM rust:latest as builder

WORKDIR /app

# Install cargo-leptos
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-leptos

# Copy manifest files
COPY Cargo.toml Cargo.toml

# Copy source code
COPY src src
COPY assets assets
COPY tailwind.css tailwind.css
COPY package.json package.json
COPY Cargo.toml Cargo.toml

# Build the Leptos app
RUN cargo leptos build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# TODO: Verify that assets or other files are required to be copied into the image, and not presented by Leptos.
# Copy built artifacts from builder
COPY --from=builder /app/target/release/palconnect_web /app/palconnect_web
COPY --from=builder /app/target/site /app/site

# Expose port
EXPOSE 3000

# Set environment variables
ENV LEPTOS_OUTPUT_NAME=palconnect_web
ENV RUST_LOG=info

# Run the app
CMD ["/app/palconnect_web"]
