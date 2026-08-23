#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# Milestone V3 — Build Hetzner Packer Golden Image
# ─────────────────────────────────────────────────────────────────────────────
set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PACKER_DIR="$PROJECT_DIR/infrastructure/v3-nginx-cluster/packer"

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

# Ensure compiled Rust release binary exists before baking image
if [ ! -f "$PROJECT_DIR/target/release/ecommerce_lab" ]; then
    echo "🦀 Rust binary missing in target/release/. Compiling release binary..."
    cd "$PROJECT_DIR"
    cargo build --release
fi


echo "🚀 Building Hetzner Cloud Golden Image with Packer..."
cd "$PACKER_DIR"


# Initialize Packer plugins if needed
packer init .

# Build the snapshot image
packer build v3-golden-image.pkr.hcl

echo "🎉 Golden Image snapshot successfully created on Hetzner Cloud!"
