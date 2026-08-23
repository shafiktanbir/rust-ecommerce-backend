# Infrastructure as Code (IaC) & Deployment Orchestration

This directory contains production Infrastructure as Code (Terraform) and Configuration Management (Ansible) playbooks for the Rust E-Commerce Scaling Lab.

## Directory Structure

```
infrastructure/
├── v3-nginx-cluster/          ← Milestone V3: Nginx LB + Axum Workers + Hetzner VPC
│   ├── terraform/             ← HCL Infrastructure provisioning (Hetzner Cloud)
│   └── ansible/               ← Systemd, Docker, and Nginx deployment playbooks
│
└── v4-background-workers/     ← Milestone V4 (Upcoming): Job Queue & Worker Nodes
```

---

## Milestone V3: Nginx Load Balancer Cluster (`v3-nginx-cluster`)

### Architecture
- **Provider**: Hetzner Cloud (or DigitalOcean)
- **Nodes**:
  - `ecommerce-nginx-lb`: Nginx Load Balancer (Private IP `10.0.1.10`)
  - `ecommerce-api-1`: Axum API Worker 1 (Private IP `10.0.1.11`)
  - `ecommerce-api-2`: Axum API Worker 2 (Private IP `10.0.1.12`)
  - `ecommerce-db`: PostgreSQL 15 + Redis 7 Data Node (Private IP `10.0.1.20`)
- **Networking**: Isolated Private VPC (`10.0.0.0/16`) + Hetzner Cloud Firewall (Public access restricted to ports 22 & 8080).

### Deployment Quickstart

#### 1. Provision Infrastructure with Terraform
```bash
cd infrastructure/v3-nginx-cluster/terraform
export HCLOUD_TOKEN="your_hetzner_api_token"
terraform init
terraform apply
```

#### 2. Deploy Services & Kernel Tuning with Ansible
```bash
cd ../ansible
ansible-playbook deploy.yml
```

#### 3. Run Benchmark from Local Machine
```bash
k6 run load-tests/v3_stress_3000vu.js -e TARGET_URL=http://<NGINX_PUBLIC_IP>:8080
```

#### 4. Destroy Infrastructure (Zero Cost)
```bash
cd ../terraform
terraform destroy
```
