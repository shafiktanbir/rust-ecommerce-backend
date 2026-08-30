#!/usr/bin/env bash
set -euo pipefail

CLUSTER_NAME="ecom-k3s"

echo "=== Teardown: Stopping & Removing k3d Cluster ($CLUSTER_NAME) ==="
if k3d cluster list | grep -q "$CLUSTER_NAME"; then
  k3d cluster delete "$CLUSTER_NAME"
  echo "Cluster $CLUSTER_NAME deleted successfully."
else
  echo "Cluster $CLUSTER_NAME does not exist."
fi
