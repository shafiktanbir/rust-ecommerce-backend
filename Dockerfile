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

# ─── Stage 3: Hardened Non-Root Runtime ─────────────────────────────────────────
FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 10001 -s /bin/bash appuser

COPY --from=builder --chown=appuser:appuser /app/target/release/ecommerce_lab /app/ecommerce_lab
COPY --from=builder --chown=appuser:appuser /app/migrations /app/migrations

USER appuser

EXPOSE 8080

HEALTHCHECK --interval=15s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["/app/ecommerce_lab"]
