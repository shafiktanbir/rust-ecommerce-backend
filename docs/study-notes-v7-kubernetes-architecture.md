# Study Notes: Milestone V7 — Kubernetes Orchestration, Docker `cargo-chef` & Production Networking

> **Topic**: Containerizing the Rust E-Commerce Scaling Lab, multi-stage build optimization, local Kubernetes (`k3d`) ingress routing, and SRE incident troubleshooting.

---

## 1. System Architecture Overview

```text
               External Client / k6 Load Tester
                              │
                              ▼
                  k3d Ingress LoadBalancer (Port 8888)
                              │
                              ▼
                  axum-api-service (ClusterIP:8080)
                              │
         ┌────────────────────┼────────────────────┐
         ▼                    ▼                    ▼
   Axum Pod 1            Axum Pod 2            Axum Pod 3
 (Liveness/Readiness)  (Liveness/Readiness)  (Liveness/Readiness)
         │                    │                    │
 ┌───────┴───────────────┬────┴────────────────────┴──────────────┐
 ▼                       ▼                                        ▼
postgres-primary     postgres-replica       redis-service    redpanda-service
(Deployment:5432)   (Deployment:5435)      (Deployment:6379) (Deployment:9092)
  [Writer Pool]        [Reader Pool]           [Cache/Jobs]    [Outbox Kafka]
```

---

## 2. Multi-Stage Docker Build Optimization (`cargo-chef`)

### A. The 3-Stage Container Pipeline
1. **Stage 1 (Planner)**: Generates a lightweight `recipe.json` containing only dependency lock metadata.
2. **Stage 2 (Builder)**: Runs `cargo chef cook --release` to compile and cache crate dependencies. This layer remains cached across application code edits.
3. **Stage 3 (Runtime)**: Copies compiled binary and database migrations into a clean `debian:bookworm-slim` base image (~25MB).

### B. Build Context Optimization & `.dockerignore`
- **Issue**: Without [.dockerignore](file:///home/shafikul/Documents/coding/research-playground-loop/rust%20ecommerse-loop/.dockerignore), Docker transfers the local `target/` directory into the build context. In Rust projects, `target/` can grow to **6GB+**, causing slow builds and `ResourceExhausted: write /app/... no space left on device` errors.
- **Solution**: Excluding `target/`, `.git/`, and `.agents/` reduced build context transfer from **5.97GB down to 21KB**, dropping build preparation time from 10 minutes to under 1 second.

---

## 3. Kubernetes SRE Troubleshooting Lessons

### Lesson 1: Offline `sqlx` Query Compilation (`SQLX_OFFLINE=true`)
- **Problem**: Building containers with `ENV SQLX_OFFLINE=true` failed when new macro queries (`outbox` or `replication lag`) were added to Rust code without updating `.sqlx/`.
- **Diagnosis**: `sqlx::query!` requires matching pre-compiled JSON query signatures in `.sqlx/`.
- **Solution**: Run `DATABASE_URL=... cargo sqlx prepare` locally against a running PostgreSQL database whenever modifying `sqlx::query!` invocations before running `docker build`.

### Lesson 2: Node Taints & Disk Pressure (`node.kubernetes.io/disk-pressure`)
- **Problem**: Pods remained stuck in `Pending` or `Evicted` status with event `0/1 nodes are available: 1 node(s) had untolerated taint(s)`.
- **Diagnosis**: Kubelet continuously monitors root disk utilization (`df -h /`). When disk usage exceeds ~85% (14GB free), Kubelet applies `node.kubernetes.io/disk-pressure:NoSchedule` taint to prevent new pod allocations.
- **Solution**:
  1. Clean up unused Docker layers (`docker system prune -af` reclaimed 10.64GB).
  2. Configure `k3d` with explicit eviction thresholds: `--k3s-arg "--kubelet-arg=eviction-hard=imagefs.available<2%,nodefs.available<2%@server:*"`.

### Lesson 3: Entrypoint vs. Command Override in Kubernetes
- **Problem**: Redpanda container failed with `cli_parser.cc: Argument parse error: unrecognised option '--mode=dev-container'`.
- **Diagnosis**: Specifying `command: ["redpanda"]` in K8s replaces the container image `ENTRYPOINT` script (`/entrypoint.sh`). Redpanda requires `/entrypoint.sh` to translate arguments properly.
- **Solution**: Pass `/entrypoint.sh` in `command` array or separate `command: ["redpanda"]` and `args: ["start", "--mode", "dev-container", ...]` in [04-redpanda.yaml](file:///home/shafikul/Documents/coding/research-playground-loop/rust%20ecommerse-loop/infrastructure/k8s/04-redpanda.yaml).

---

## 4. Ingress Networking & Host Access

### Why `curl http://localhost:8888/health` Works Without `kubectl port-forward`

```text
Host `curl http://localhost:8888/health`
  └─► Docker Port Forwarding (8888:80)
        └─► k3d LoadBalancer Node (`k3d-ecom-k3s-serverlb`)
              └─► K8s Ingress Controller (Traefik / Nginx on Port 80)
                    └─► Ingress Routing Rule (`06-ingress.yaml` path "/")
                          └─► Service ClusterIP (`axum-api-service:8080`)
                                └─► Pod Replica (`axum-api`)
```

- **`kubectl port-forward`**: Direct point-to-point debug tunnel to a single pod.
- **`k3d Ingress LoadBalancer`**: Production-grade ingress architecture routing traffic through a host-mapped port into the Ingress Controller and Service mesh.
