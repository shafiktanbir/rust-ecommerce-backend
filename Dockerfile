# ─── Stage 1: Chef Planner ───────────────────────────────────────────────────
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ─── Stage 2: Chef Builder ───────────────────────────────────────────────────
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - cached layer across code edits
RUN cargo chef cook --release --recipe-path recipe.json
# Copy app source & build final binary
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin ecommerce_lab

# ─── Stage 3: Runtime ────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ecommerce_lab /app/ecommerce_lab
COPY migrations /app/migrations

EXPOSE 8080 8081 8082 8083

CMD ["/app/ecommerce_lab"]

