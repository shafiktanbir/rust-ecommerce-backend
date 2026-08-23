# ─── Fast Production Dockerfile ──────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy pre-compiled release binary and database migrations
COPY target/release/ecommerce_lab /app/ecommerce_lab
COPY migrations /app/migrations

EXPOSE 8080 8081 8082 8083

CMD ["/app/ecommerce_lab"]
