# Stage 1: Chef - prepare recipe for dependency caching
FROM lukemathwalker/cargo-chef:latest-rust-1.85 AS chef
WORKDIR /app

# Stage 2: Planner - analyze dependencies
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder - build dependencies (cached) then app
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this layer is cached unless Cargo.toml/Cargo.lock change
RUN cargo chef cook --release --recipe-path recipe.json

# Build the actual application
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Stage 4: Runtime - minimal final image
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Non-root user for security
RUN groupadd -g 1000 bot && \
    useradd -u 1000 -g bot -s /bin/sh bot

RUN mkdir -p /app/data && chown -R bot:bot /app
WORKDIR /app

COPY --from=builder /app/target/release/button-attach-bot /app/button-attach-bot
RUN chown bot:bot /app/button-attach-bot

USER bot

# NO secrets here - all from environment at runtime
ENV RUST_LOG=info
ENV DATABASE_URL=/app/data/bot.db

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pgrep button-attach-bot || exit 1

CMD ["/app/button-attach-bot"]
