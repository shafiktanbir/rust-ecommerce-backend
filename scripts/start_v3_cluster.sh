#!/usr/bin/env bash
set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

# Set open file descriptor limit to handle high-concurrency TCP sockets
ulimit -n 65535 2>/dev/null || ulimit -n 4096 2>/dev/null || true

echo "🚀 Starting Milestone V3 Multi-Instance Axum Cluster..."

# 1. Clean stop existing cluster processes
bash "$PROJECT_DIR/scripts/stop_v3_cluster.sh" 2>/dev/null || true

# 2. Verify release binary exists
if [ ! -f "$PROJECT_DIR/target/release/ecommerce_lab" ]; then
    echo "📦 Building release binary..."
    cargo build --release
fi

# 3. Launch 3 parallel Axum API workers
echo "🟢 Launching API Worker 1 on port 8081..."
nohup env APP_PORT=8081 ./target/release/ecommerce_lab </dev/null > /tmp/ecommerce_api_8081.log 2>&1 &
echo $! > /tmp/ecommerce_api_8081.pid

echo "🟢 Launching API Worker 2 on port 8082..."
nohup env APP_PORT=8082 ./target/release/ecommerce_lab </dev/null > /tmp/ecommerce_api_8082.log 2>&1 &
echo $! > /tmp/ecommerce_api_8082.pid

echo "🟢 Launching API Worker 3 on port 8083..."
nohup env APP_PORT=8083 ./target/release/ecommerce_lab </dev/null > /tmp/ecommerce_api_8083.log 2>&1 &
echo $! > /tmp/ecommerce_api_8083.pid

# Detach processes from parent shell
disown -a 2>/dev/null || true

# Give workers 2 seconds to bind sockets and run DB migrations
sleep 2

# 4. Verify individual worker health
curl -s http://127.0.0.1:8081/health > /dev/null && echo "  └─ ✅ Worker 1 (8081) Healthy"
curl -s http://127.0.0.1:8082/health > /dev/null && echo "  └─ ✅ Worker 2 (8082) Healthy"
curl -s http://127.0.0.1:8083/health > /dev/null && echo "  └─ ✅ Worker 3 (8083) Healthy"

# 5. Launch Nginx Load Balancer on port 8080
echo "🔀 Launching Nginx Load Balancer on port 8080..."
nginx -c "$PROJECT_DIR/nginx/nginx.conf"

sleep 1

# 6. Verify Nginx health check proxying
if curl -s http://127.0.0.1:8080/health | grep -q "ok"; then
    echo "🎉 V3 Cluster Online! Nginx proxying http://127.0.0.1:8080 → (8081, 8082, 8083)"
else
    echo "❌ Error launching Nginx Load Balancer"
    exit 1
fi
