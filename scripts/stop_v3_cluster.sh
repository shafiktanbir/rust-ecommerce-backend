#!/usr/bin/env bash

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "🛑 Stopping V3 Multi-Instance Cluster..."

# Kill Nginx load balancer instance
if [ -f /tmp/nginx_ecommerce.pid ]; then
    nginx -s stop -c "$PROJECT_DIR/nginx/nginx.conf" 2>/dev/null || true
fi
pkill -f "nginx" 2>/dev/null || true
rm -f /tmp/nginx_ecommerce.pid 2>/dev/null || true

# Kill worker processes
pkill -f "ecommerce_lab" 2>/dev/null || true
rm -f /tmp/ecommerce_api_*.pid 2>/dev/null || true

echo "✅ All V3 workers and Nginx stopped."
