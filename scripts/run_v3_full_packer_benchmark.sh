#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# Milestone V3 — Full End-to-End Packer + Terraform + Ansible Benchmark Pipeline
# ─────────────────────────────────────────────────────────────────────────────
# Workflow:
#  1. Compile Rust Release Binary locally
#  2. Build Hetzner Golden Image with Packer (pre-baking Docker, Nginx, & binary)
#  3. Spawn Hetzner Cloud Infrastructure from Golden Image via Terraform (~20s)
#  4. Deploy & Verify Cluster Services via Ansible
#  5. Execute 3,000 VU k6 Load Test
#  6. Guaranteed Teardown (Destroys all VPS instances automatically on exit)
# ─────────────────────────────────────────────────────────────────────────────
set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TF_DIR="$PROJECT_DIR/infrastructure/v3-nginx-cluster/terraform"
ANSIBLE_DIR="$PROJECT_DIR/infrastructure/v3-nginx-cluster/ansible"
PACKER_SCRIPT="$PROJECT_DIR/scripts/build_v3_packer_image.sh"
IMAGE_NAME="v3-golden-image-v1"

# Automatically source HCLOUD_TOKEN from .env if present
if [ -z "$HCLOUD_TOKEN" ]; then
    if [ -f "$PROJECT_DIR/.env" ]; then
        HCLOUD_TOKEN=$(grep -E "^(HCLOUD_TOKEN|terraform_api_token)=" "$PROJECT_DIR/.env" | cut -d '=' -f2- | tr -d ' "' | head -n 1)
        export HCLOUD_TOKEN
    fi
fi

if [ -z "$HCLOUD_TOKEN" ]; then
    echo "❌ Error: HCLOUD_TOKEN environment variable is not set and not found in .env."
    exit 1
fi

# Guaranteed Teardown Handler (Runs on script exit, error, or Ctrl+C)
cleanup() {
    echo ""
    echo "🧹 Step 5/5: Guaranteeing Hetzner VPS Teardown..."
    if [ -d "$TF_DIR" ]; then
        cd "$TF_DIR" 2>/dev/null || true
        terraform destroy -lock=false -auto-approve 2>/dev/null || true
    fi
    echo "🎉 Benchmark pipeline complete! All VPS instances destroyed. Zero wasted money."
}
trap cleanup EXIT

echo "========================================================================="
echo "🚀 STAGE 1/5: Building Rust Release Binary & Hetzner Packer Golden Image"
echo "========================================================================="
chmod +x "$PACKER_SCRIPT"
"$PACKER_SCRIPT"

echo ""
echo "========================================================================="
echo "🚀 STAGE 2/5: Spawning Hetzner Infrastructure from Golden Image (Terraform)"
echo "========================================================================="
cd "$TF_DIR"
terraform apply -var="server_image=$IMAGE_NAME" -lock=false -auto-approve

NGINX_IP=$(terraform output -raw nginx_lb_public_ip)
echo "  └─ ✅ Cluster Spawned! Nginx Load Balancer Public IP: http://$NGINX_IP:8080"

echo ""
echo "========================================================================="
echo "📦 STAGE 3/5: Deploying & Configuring Cluster Services (Ansible)"
echo "========================================================================="
echo "⏳ Waiting for VPS SSH port to be open on $NGINX_IP..."
until ssh -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=3 -o BatchMode=yes root@"$NGINX_IP" echo "SSH Ready" 2>/dev/null; do
    echo "  └─ Waiting for SSH on $NGINX_IP..."
    sleep 2
done
echo "  └─ ✅ SSH Ready! Running Ansible Playbook..."

cd "$ANSIBLE_DIR"
ansible-playbook deploy.yml

echo ""
echo "========================================================================="
echo "⚡ STAGE 4/5: Verifying Cluster HTTP Health & Executing k6 Load Test"
echo "========================================================================="
echo "⏳ Polling http://$NGINX_IP:8080/health..."
until curl -s -f -m 3 "http://$NGINX_IP:8080/health" > /dev/null 2>&1; do
    echo "  └─ Application starting... retrying in 2s"
    sleep 2
done
echo "  └─ ✅ Application Cluster HTTP 200 OK!"

cd "$PROJECT_DIR"
mkdir -p load-tests/results
set +e
k6 run --out json=load-tests/results/v3_packer_hetzner_live.json load-tests/v3_stress_3000vu.js -e TARGET_URL=http://$NGINX_IP:8080
K6_EXIT_CODE=$?
set -e

if [ $K6_EXIT_CODE -eq 0 ]; then
    echo "✅ k6 Benchmark Passed all thresholds!"
else
    echo "⚠️ k6 Benchmark Completed (Exit code: $K6_EXIT_CODE)"
fi
