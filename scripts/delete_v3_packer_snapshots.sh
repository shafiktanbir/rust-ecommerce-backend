#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# Milestone V3 — Delete Custom Hetzner Packer Golden Image Snapshots
# ─────────────────────────────────────────────────────────────────────────────
set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

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

echo "🔍 Querying Hetzner Cloud API for custom snapshots..."

# Fetch all custom snapshots matching project=rust-ecommerce-lab
RESPONSE=$(curl -s -H "Authorization: Bearer $HCLOUD_TOKEN" "https://api.hetzner.cloud/v1/images?type=snapshot")

# Extract image IDs matching our project tag or description
SNAPSHOT_IDS=$(echo "$RESPONSE" | jq -r '.images[] | select(.labels.project=="rust-ecommerce-lab" or .description=="v3-golden-image-v1") | .id')

if [ -z "$SNAPSHOT_IDS" ]; then
    echo "🎉 No active custom Packer snapshots found on Hetzner Cloud! (Storage Cost: €0.00)"
    exit 0
fi

echo "FOUND SNAPSHOTS TO DELETE:"
echo "$SNAPSHOT_IDS"
echo ""

for SNAPSHOT_ID in $SNAPSHOT_IDS; do
    echo "🧹 Deleting Hetzner Cloud Snapshot ID: $SNAPSHOT_ID..."
    curl -s -X DELETE -H "Authorization: Bearer $HCLOUD_TOKEN" "https://api.hetzner.cloud/v1/images/$SNAPSHOT_ID"
    echo "  └─ ✅ Snapshot $SNAPSHOT_ID deleted!"
done

echo ""
echo "🎉 All custom Packer Golden Image snapshots removed. Zero recurring storage charges."
