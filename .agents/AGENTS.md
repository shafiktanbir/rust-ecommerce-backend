# Agent Behavior Guidelines - Rust High-Throughput E-Commerce Lab

## hetzner-packer-benchmark-strategy-rules

Whenever operating in this repository (`rust ecommerse-loop`) or running Hetzner Cloud infrastructure benchmarks:

1. **Pre-baked Packer Golden Image Strategy**:
   - Always use Packer (`infrastructure/v3-nginx-cluster/packer/v3-golden-image.pkr.hcl`) to pre-bake Ubuntu 24.04 OS, Docker, Postgres 15 & Redis 7 container images, Nginx, kernel sysctl tuning (`net.core.somaxconn=65535`), and the compiled Rust binary into a Hetzner Cloud Snapshot (`v3-golden-image-v1`).
   - Avoid dynamic runtime provisioning with Ansible (`apt update`, SSH file copies) on every `terraform apply` spawn loop to keep VM boot times under **30 seconds**.

2. **Benchmark Pipeline Scripts**:
   - Master Pipeline: `./scripts/run_v3_full_packer_benchmark.sh` (Compiles Rust binary, builds Packer snapshot, spawns Terraform VPS, verifies HTTP health, runs k6 test, and auto-destroys).
   - Image Builder: `./scripts/build_v3_packer_image.sh`
   - Fast Benchmark Runner: `./scripts/run_v3_hetzner_benchmark.sh` (supports `SKIP_ANSIBLE=true`)

3. **Cost-Saving & Guaranteed Teardown Protocol**:
   - Hetzner Cloud snapshots are billed pro-rated hourly based on compressed storage size (~€0.0143/GB/month, i.e., ~€0.00005/hr for a 3GB image).
   - Temporary Packer build instances run for ~3 mins (~€0.0003 cost) and are destroyed immediately upon snapshot creation.
   - Always ensure auto-teardown of benchmark VPS instances (`terraform destroy -auto-approve`) post-k6 benchmark to maintain zero idle compute costs.

4. **HTTP Health Check Polling**:
   - Poll `http://$NGINX_IP:8080/health` directly with backoff retries rather than relying solely on SSH port loops.
