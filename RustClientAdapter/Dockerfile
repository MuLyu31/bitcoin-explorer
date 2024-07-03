# Build# Build stage
FROM rust:1.74 as builder
WORKDIR /usr/src/app

# Copy over your manifests
COPY Cargo.toml .
RUN touch Cargo.lock

# This is a dummy build to get the dependencies cached
RUN mkdir src && \
    echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Now copy in your actual source code
COPY . .

# This builds your application using the cached dependencies
RUN cargo build --release

# Runtime stage
FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    libpq-dev \
    ca-certificates \
    libssl3 \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy the built application
COPY --from=builder /usr/src/app/target/release/rust_client_adapter .

# Copy the SQL init script
COPY ./sql/init.sql /usr/src/app/init.sql

EXPOSE 3001

# Set the startup command
CMD ["./rust_client_adapter"]rust_client_adapter"]
