#!/usr/bin/env bash

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "🛑 Stopping V3 Multi-Instance Cluster..."

# Kill Nginx load balancer instance
pkill -9 -f "nginx.*nginx.conf" 2>/dev/null || true
rm -f /tmp/nginx_ecommerce.pid 2>/dev/null || true

# Kill worker processes
pkill -9 -f "ecommerce_lab" 2>/dev/null || true
rm -f /tmp/ecommerce_api_*.pid 2>/dev/null || true

echo "✅ All V3 workers and Nginx stopped."
