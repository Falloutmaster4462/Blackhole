FROM rust:latest as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests and source
COPY Cargo.toml ./
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/blackhole-mail /app/blackhole-mail
COPY --from=builder /app/migrations /app/migrations

# Create storage directory
RUN mkdir -p /var/lib/blackhole/storage

EXPOSE 8080 2525

CMD ["/app/blackhole-mail"]
