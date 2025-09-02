# 📌 Pillar 7: Containerization, `cargo-chef` & Kubernetes Operations (Posts 181 - 210)

> Technical content series focused on Docker container optimization for Rust, multi-stage `cargo-chef` layer caching, `.dockerignore` disk space recovery, Kubernetes orchestration (`k3d`), declarative manifests, and readiness/liveness probes.

---

## Post 181: Cutting Rust Docker Build Times from 8 Minutes to 25 Seconds with `cargo-chef`
* **Target Audience**: CTOs, DevOps Leads, Rust Developers, CI/CD Engineers.
* **Viral Hook**: "Rebuilding 300+ Rust dependencies on every 1-line code change is destroying your CI/CD pipeline. How `cargo-chef` caches compiled layers."
* **Core Problem**: Standard Dockerfiles copy source code before `cargo build`, invalidating Docker's layer cache on every code update and forcing full dependency re-compilation.
* **Technical 3-Stage Dockerfile**:
  ```dockerfile
  # Stage 1: Recipe Generator
  FROM lukemathwalker/cargo-chef:latest-rust-1.80 AS chef
  WORKDIR /app
  COPY . .
  RUN cargo chef prepare --recipe-path recipe.json

  # Stage 2: Cache Dependency Layers
  FROM chef AS builder
  COPY --from=chef /app/recipe.json recipe.json
  RUN cargo chef cook --release --recipe-path recipe.json
  COPY . .
  RUN cargo build --release --bin ecommerce-lab-backend

  # Stage 3: Tiny Debian Runtime
  FROM debian:bookworm-slim AS runtime
  WORKDIR /app
  COPY --from=builder /app/target/release/ecommerce-lab-backend .
  CMD ["./ecommerce-lab-backend"]
  ```
* **Metrics Impact**: CI/CD container build time dropped from 8 minutes down to **25 seconds (19x speedup)** on code changes.
* **Cold Outreach DM**: "Hey [Name], saw [Company] is running Rust microservices in Docker/Kubernetes. Using `cargo-chef` to cache compiled Rust dependency layers in Docker dropped our image build time from 8 minutes to 25 seconds. Shared our 3-stage Dockerfile here!"

---

## Post 182: Reclaiming 10.6 GB Disk Space with `.dockerignore` — Preventing Target Directory Copy
* **Target Audience**: DevOps Engineers, Lead Developers.
* **Viral Hook**: "Why our `docker build` context upload was taking 3 minutes and transferring 10.6 Gigabytes of useless local build artifacts."
* **Core Problem**: Forgetting to add `.dockerignore` copies the local 10GB `target/` build directory into the Docker daemon context during build initialization.
* **Technical `.dockerignore` Configuration**:
  ```gitignore
  target/
  .git/
  .env
  node_modules/
  *.log
  ```
* **Metrics Impact**: Docker build context payload reduced from **10.64 GB down to 240 KB (-99.99% drop)**; context transfer time dropped from 3 minutes to 0.1 seconds.
* **Cold Outreach DM**: "Hey [Name], saw your update on Docker build optimization. Adding a targeted `.dockerignore` to exclude local Rust `target/` folders reclaimed 10.6GB of Docker disk space and eliminated context transfer delays. Shared our `.dockerignore` template here!"

---

## Post 183: Kubernetes Readiness vs Liveness Probes — Preventing Traffic Routing to Crashing Pods
* **Target Audience**: Head of Infrastructure, Lead SREs, Kubernetes Operators.
* **Viral Hook**: "Why setting `livenessProbe` to query your database crashes your Kubernetes cluster during database maintenance. The Probe Isolation Rule."
* **Core Difference**:
  * **Liveness Probe (`/health`)**: Checks if the API process is alive. Fails $\rightarrow$ K8s **restarts the pod**.
  * **Readiness Probe (`/health/db`)**: Checks if the pod can serve traffic. Fails $\rightarrow$ K8s **removes pod from Ingress endpoints** (no restart).
* **Technical Manifest Snippet**:
  ```yaml
  livenessProbe:
    httpGet:
      path: /health
      port: 8080
    initialDelaySeconds: 5
    periodSeconds: 10
  readinessProbe:
    httpGet:
      path: /health/db
      port: 8080
    initialDelaySeconds: 10
    periodSeconds: 5
  ```
* **Metrics Impact**: Eliminated cascading pod restart loops during temporary database maintenance windows; 100% pod traffic isolation.
* **Cold Outreach DM**: "Hey [Name], saw your post on Kubernetes health check design. Separating process liveness (`/health`) from database readiness (`/health/db`) prevents K8s from entering destructive pod restart loops during DB maintenance. Shared our K8s manifest config here!"

---

## Post 184: Declarative Kubernetes Manifest Structure — Organizing Clean K8s Trees
* **Target Audience**: DevOps Managers, Principal Engineers.
* **Viral Hook**: "How we structured our Kubernetes infrastructure manifests into an ordered, numbered tree for 1-command deployment (`kubectl apply -f infrastructure/k8s/`)."
* **Directory Tree**:
  ```text
  infrastructure/k8s/
  ├── 00-namespace.yaml          (Namespace isolation)
  ├── 01-configmap-secrets.yaml  (Environment config & credentials)
  ├── 02-postgres.yaml           (Primary & Replica deployments)
  ├── 03-redis.yaml              (Cache & Job Queue)
  ├── 04-redpanda.yaml           (Kafka event broker)
  ├── 05-axum-api.yaml           (API Worker deployment with probes)
  └── 06-ingress.yaml            (Ingress controller routing)
  ```
* **Metrics Impact**: Reduced infrastructure deployment time to a single deterministic command; 100% reproducible cluster state.
* **Cold Outreach DM**: "Hey [Name], saw [Company] is refining Kubernetes manifest architecture. Organizing K8s manifests into a numbered sequence (`00-namespace` to `06-ingress`) ensures deterministic resource creation order during automated deployments. Shared our manifest layout here!"

---

## Post 185: Kubernetes Disk Eviction (`eviction-hard`) — Tuning Local K3s Clusters for Storage Limits
* **Target Audience**: DevOps Engineers, Local Dev Environment Leads.
* **Viral Hook**: "Why our local `k3d` Kubernetes cluster randomly evicted API pods with `DiskPressure`. How to adjust eviction thresholds."
* **Core Problem**: Default Kubernetes settings trigger pod evictions when node available disk space falls below 15%, causing local development clusters on small drives to freeze.
* **Technical Fix (`deploy-k3d.sh` flag)**:
  ```bash
  # Override default K3s disk eviction threshold for local development
  k3d cluster create ecommerce-cluster \
    --k3s-arg "--kubelet-arg=eviction-hard=imagefs.available<2%,nodefs.available<2%@server:0"
  ```
* **Metrics Impact**: Eliminated false-positive local pod evictions; maintained 100% cluster stability on local dev laptops.
* **Cold Outreach DM**: "Hey [Name], saw your post on local Kubernetes (`k3d`/`k3s`) development environments. Tuning `eviction-hard < 2%` in kubelet flags prevents unexpected pod evictions on developer laptops with limited disk space. Shared our `k3d` setup script here!"

---

## Post 186: Securing Rust Runtime Containers — Non-Root Users in Debian-Slim Docker Images
* **Target Audience**: Security Officers, DevSecOps Engineers, CTOs.
* **Viral Hook**: "Running Docker containers as `root` gives attackers root access to your container environment. How to run non-root Rust binaries."
* **Core Problem**: Default Docker runtime containers execute binaries as PID 1 root user, violating DevSecOps principle of least privilege.
* **Technical Dockerfile Snippet**:
  ```dockerfile
  # Create unprivileged runtime user
  RUN adduser --disabled-password --gecos "" appuser
  USER appuser
  EXPOSE 8080
  CMD ["./ecommerce-lab-backend"]
  ```
* **Metrics Impact**: Achieved 100% compliance with Kubernetes Non-Root Security Context policies (`runAsNonRoot: true`).
* **Cold Outreach DM**: "Hey [Name], saw your update on container security & Kubernetes hardening. Enforcing unprivileged `appuser` execution in Debian runtime containers guarantees compliance with K8s security policies without impacting performance. Shared our non-root Dockerfile here!"

---

## Post 187: Pod CPU Throttling in Kubernetes — Understanding `resources.limits` vs `requests`
* **Target Audience**: Lead SREs, Performance Engineers, CTOs.
* **Viral Hook**: "Why setting hard `resources.limits.cpu` in Kubernetes causes latency spikes due to CFS quota throttling even when CPU usage is low."
* **Core Problem**: Linux Completely Fair Scheduler (CFS) throttles CPU cycles in 100ms quota windows if a pod exceeds its CPU limit in bursts, causing 500ms API latency spikes.
* **Technical Rule**: Set `resources.requests.cpu` for scheduling, but omit or raise `resources.limits.cpu` for latency-critical API workers!
* **Metrics Impact**: Eliminated CFS CPU throttling spikes; API latency under 2,000 VUs dropped from 650ms to **103ms**.
* **Cold Outreach DM**: "Hey [Name], saw your discussion on Kubernetes pod resource limits. Removing hard CPU limits while keeping CPU requests configured prevents Linux CFS quota throttling from causing artificial latency spikes on API workers. Shared our resource tuning metrics here!"

---

## Post 188: Micro-Containers with Alpine vs Debian-Slim — Solving Glibc & OpenSSL Dynamic Linking Issues
* **Target Audience**: DevOps Engineers, Systems Developers.
* **Viral Hook**: "Why your Rust binary built on Ubuntu fails to start in Alpine Linux with `No such file or directory`. How to choose the right base image."
* **Core Problem**: Rust dynamically links against `glibc` by default. Alpine Linux uses `musl`, causing dynamic linker failure unless compiled specifically for `x86_64-unknown-linux-musl`.
* **Metrics Comparison**:
  * `debian:bookworm-slim` (80MB image size, zero glibc/OpenSSL compatibility issues) $\rightarrow$ Recommended!
  * `alpine` (12MB image size, requires musl toolchain cross-compilation).
* **Cold Outreach DM**: "Hey [Name], saw your post on Rust container base images. Using `debian:bookworm-slim` avoids musl/glibc cross-compilation headaches while keeping final runtime image size under 80MB. Documented our container base image comparison here!"

---

## Post 189: Kubernetes ConfigMaps and Secrets Injection — Managing Environment Variables Safely
* **Target Audience**: Cloud Engineers, DevSecOps Specialists.
* **Viral Hook**: "Never hardcode database passwords in Kubernetes deployments. Declarative `Secret` and `ConfigMap` injection."
* **Technical Manifest Snippet**:
  ```yaml
  envFrom:
    - configMapRef:
        name: app-config
    - secretRef:
        name: app-secrets
  ```
* **Metrics Impact**: 100% environment variable isolation; zero secret leakage in Git repositories or container layers.
* **Cold Outreach DM**: "Hey [Name], saw your update on GitOps & K8s secret management. Declarative `ConfigMap` and `Secret` injection decouples environment configurations from application deployment manifests. Shared our K8s secret management template here!"

---

## Post 190: Ingress Controller Nginx Annotations — Tuning Proxy Buffers and Timeouts in K8s
* **Target Audience**: Kubernetes Network Engineers, SRE Leads.
* **Viral Hook**: "Why large file uploads throw `413 Request Entity Too Large` in Kubernetes Ngress. The Annotations Cheat Sheet."
* **Technical Annotations**:
  ```yaml
  metadata:
    annotations:
      nginx.ingress.kubernetes.io/proxy-body-size: "10m"
      nginx.ingress.kubernetes.io/proxy-connect-timeout: "15"
      nginx.ingress.kubernetes.io/proxy-read-timeout: "60"
  ```
* **Metrics Impact**: Eliminated 413 and 504 errors on Ingress proxy layers under heavy file upload benchmarks.
* **Cold Outreach DM**: "Hey [Name], saw [Company] is tuning Kubernetes Nginx Ingress controllers. Configuring explicit proxy buffer and timeout annotations prevents gateway timeouts on heavy API payloads. Shared our Ingress annotation template here!"

---

## Posts 191 - 210 Overview (Summary Matrix in Detailed File)
* **Post 191**: Kubernetes Horizontal Pod Autoscaler (HPA) — Scaling on Custom Prometheus Metrics.
* **Post 192**: Zero-Downtime Rolling Updates in K8s — Tuning `maxSurge` and `maxUnavailable`.
* **Post 193**: Pod Topology Spread Constraints — Distributing API Pods Across Availability Zones.
* **Post 194**: Managing Stateful Sets vs Deployments for PostgreSQL in Kubernetes.
* **Post 195**: Helm Charts vs Kustomize — Choosing the Right Manifest Templating Strategy.
* **Post 196**: Distroless Docker Images for Rust — Building 20MB Security-Hardened Containers.
* **Post 197**: Kubernetes Init Containers — Polling PostgreSQL Readiness Before API Worker Boot.
* **Post 198**: Monitoring Pod Memory Usage (`container_memory_working_set_bytes`) to Prevent OOMKills.
* **Post 199**: Securing Inter-Pod Network Traffic with Calico Network Policies.
* **Post 200**: Multi-Stage Docker Caching in GitHub Actions & GitLab CI Pipelines.
* **Post 201**: Managing Persistent Volume Claims (PVC) Performance in Kubernetes.
* **Post 202**: Pod Disruption Budgets (PDB) — Ensuring High Availability During Node Drains.
* **Post 203**: Docker Buildx Cross-Platform Compiling (ARM64 vs AMD64) for Cloud Deployments.
* **Post 204**: Kubernetes Namespace Resource Quotas — Preventing Rogue Pods from Starving Clusters.
* **Post 205**: Debugging Pod Crashing Loops (`CrashLoopBackOff`) with `kubectl logs --previous`.
* **Post 206**: Building Small Rust Containers with Strip Symbols (`strip = true`) and UPX Compression.
* **Post 207**: Service Mesh Overview: Istio vs Linkerd for Rust Microservices.
* **Post 208**: Managing Environment-Specific Config Overlays with Kustomize.
* **Post 209**: Container Security Scanning with Trivy in CI/CD Pipelines.
* **Post 210**: The Complete Kubernetes Production Readiness Checklist for Senior Engineers.
