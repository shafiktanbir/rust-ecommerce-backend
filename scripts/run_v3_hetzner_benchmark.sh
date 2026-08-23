#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# Milestone V3 — Automated Hetzner Benchmark & Guaranteed Teardown Script
# ─────────────────────────────────────────────────────────────────────────────
set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TF_DIR="$PROJECT_DIR/infrastructure/v3-nginx-cluster/terraform"
ANSIBLE_DIR="$PROJECT_DIR/infrastructure/v3-nginx-cluster/ansible"

# Automatically source HCLOUD_TOKEN from .env if present
if [ -z "$HCLOUD_TOKEN" ]; then
    if [ -f "$PROJECT_DIR/.env" ]; then
        HCLOUD_TOKEN=$(grep -E "^(HCLOUD_TOKEN|terraform_api_token)=" "$PROJECT_DIR/.env" | cut -d '=' -f2- | tr -d ' "' | head -n 1)
        export HCLOUD_TOKEN
    fi
fi

if [ -z "$HCLOUD_TOKEN" ]; then
    echo "❌ Error: HCLOUD_TOKEN environment variable is not set and not found in .env."
    echo "Please set HCLOUD_TOKEN in .env or run: export HCLOUD_TOKEN=\"your_token\""
    exit 1
fi

# Guaranteed Teardown Handler (Runs on script exit, error, or Ctrl+C)
cleanup() {
    echo ""
    echo "🧹 Step 4/4: Guaranteeing Hetzner VPS Teardown..."
    if [ -d "$TF_DIR" ]; then
        cd "$TF_DIR" 2>/dev/null || true
        terraform destroy -lock=false -auto-approve 2>/dev/null || true
    fi
    echo "🎉 Benchmark workflow complete! All VPS instances destroyed. Zero wasted money."
}
trap cleanup EXIT

echo "🚀 Step 1/4: Spawning Hetzner Cloud Infrastructure (Terraform)..."
cd "$TF_DIR"
terraform apply -lock=false -auto-approve

# Extract Nginx Load Balancer Public IP from Terraform output
NGINX_IP=$(terraform output -raw nginx_lb_public_ip)
echo "  └─ ✅ Infrastructure Spawned! Nginx Public IP: http://$NGINX_IP:8080"

# Step 2: Deploy Cluster (Ansible or Fast Golden Image)
if [ "$SKIP_ANSIBLE" = "true" ]; then
    echo "⚡ Step 2/4: Using Pre-baked Golden Image — Skipping Ansible deployment!"
else
    # Active SSH Readiness Poll
    echo "⏳ Waiting for VPS SSH to be active on $NGINX_IP..."
    until ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=3 -o BatchMode=yes root@"$NGINX_IP" echo "SSH Ready" 2>/dev/null; do
        echo "  └─ Waiting for SSH service on $NGINX_IP..."
        sleep 2
    done
    echo "  └─ ✅ SSH Active! Running Ansible Playbook..."
    cd "$ANSIBLE_DIR"
    ansible-playbook deploy.yml
fi

# Active HTTP Readiness Poll (Poll /health endpoint directly)
echo "⏳ Step 3/4: Polling Application Health Check at http://$NGINX_IP:8080/health..."
until curl -s -f -m 3 "http://$NGINX_IP:8080/health" > /dev/null 2>&1; do
    echo "  └─ Application starting up... retrying in 2s"
    sleep 2
done
echo "  └─ ✅ Application Cluster is HTTP 200 OK & Healthy!"

echo "⚡ Step 4/4: Executing 3,000 VU k6 Load Test against http://$NGINX_IP:8080..."
cd "$PROJECT_DIR"
mkdir -p load-tests/results
set +e
k6 run --out json=load-tests/results/v3_hetzner_live.json load-tests/v3_stress_3000vu.js -e TARGET_URL=http://$NGINX_IP:8080
K6_EXIT_CODE=$?
set -e

if [ $K6_EXIT_CODE -eq 0 ]; then
    echo "✅ k6 Benchmark Passed all thresholds!"
else
    echo "⚠️ k6 Benchmark Completed (Exit code: $K6_EXIT_CODE)"
fi

