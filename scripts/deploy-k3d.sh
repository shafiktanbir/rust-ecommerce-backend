#!/usr/bin/env bash
set -euo pipefail

CLUSTER_NAME="ecom-k3s"

echo "=== Step 1: Provisioning k3d Cluster ($CLUSTER_NAME) ==="
if k3d cluster list | grep -q "$CLUSTER_NAME"; then
  echo "Cluster $CLUSTER_NAME already exists. Starting if stopped..."
  k3d cluster start "$CLUSTER_NAME" || true
else
  k3d cluster create "$CLUSTER_NAME" \
    --port "8888:80@loadbalancer" \
    --k3s-arg "--kubelet-arg=eviction-hard=imagefs.available<2%,nodefs.available<2%@server:*"
fi


echo "=== Step 2: Building Docker Container Image (cargo-chef) ==="
docker build -t ecommerce_lab:v7 .

echo "=== Step 3: Importing Image into k3d Cluster ==="
k3d image import ecommerce_lab:v7 -c "$CLUSTER_NAME"

echo "=== Step 4: Applying Kubernetes Manifests ==="
kubectl apply -f infrastructure/k8s/00-namespace.yaml
kubectl apply -f infrastructure/k8s/01-configmap-secrets.yaml
kubectl apply -f infrastructure/k8s/02-postgres.yaml
kubectl apply -f infrastructure/k8s/03-redis.yaml
kubectl apply -f infrastructure/k8s/04-redpanda.yaml
kubectl apply -f infrastructure/k8s/05-axum-api.yaml
kubectl apply -f infrastructure/k8s/06-ingress.yaml

echo "=== Step 5: Waiting for Deployment Rollout ==="
kubectl rollout status deployment/axum-api -n ecommerce-lab --timeout=180s

echo "=== Deployment Complete ==="
echo "Access API at: http://localhost:8888/health"
echo "To stop and delete the cluster, run: ./scripts/teardown-k3d.sh"

