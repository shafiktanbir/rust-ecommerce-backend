# ─────────────────────────────────────────────────────────────────────────────
# Milestone V3 — Packer Golden Image Template for Hetzner Cloud
# ─────────────────────────────────────────────────────────────────────────────
packer {
  required_plugins {
    hcloud = {
      version = ">= 1.1.1"
      source  = "github.com/hetznercloud/hcloud"
    }
  }
}

variable "hcloud_token" {
  type      = string
  sensitive = true
  default   = env("HCLOUD_TOKEN")
}

variable "image_name" {
  type    = string
  default = "v3-golden-image-v1"
}

source "hcloud" "ubuntu_golden" {
  token         = var.hcloud_token
  image         = "ubuntu-24.04"
  location      = "fsn1"
  server_type   = "cpx11" # Micro build server (~€0.006/hr)
  ssh_username  = "root"
  snapshot_name = var.image_name
  snapshot_labels = {
    project = "rust-ecommerce-lab"
    tier    = "v3-nginx-cluster"
  }
}

build {
  sources = ["source.hcloud.ubuntu_golden"]


  # Step 1: Tune OS Kernel & Network Stack for High Concurrency (3,000+ VU)
  provisioner "shell" {
    inline = [
      "echo '⚡ Tuning Linux sysctl kernel parameters...'",
      "sysctl -w net.core.somaxconn=65535",
      "sysctl -w net.ipv4.tcp_max_syn_backlog=65535",
      "sysctl -w fs.file-max=2097152",
      "echo 'net.core.somaxconn = 65535' >> /etc/sysctl.conf",
      "echo 'net.ipv4.tcp_max_syn_backlog = 65535' >> /etc/sysctl.conf",
      "echo 'fs.file-max = 2097152' >> /etc/sysctl.conf"
    ]
  }

  # Step 2: Install Docker, Nginx, and Core Dependencies
  provisioner "shell" {
    inline = [
      "echo '📦 Installing Docker & Nginx...'",
      "apt-get update -y",
      "apt-get install -y docker.io docker-compose-v2 nginx curl jq git",
      "systemctl enable docker",
      "systemctl enable nginx"
    ]
  }

  # Step 3: Pre-pull Docker Container Images to cache on disk (Zero download wait on boot)
  provisioner "shell" {
    inline = [
      "echo '🐳 Pre-pulling Postgres and Redis Docker images...'",
      "docker pull postgres:15-alpine",
      "docker pull redis:7-alpine"
    ]
  }

  # Step 4: Create App Workspace Directories & Upload Rust Binary
  provisioner "shell" {
    inline = [
      "mkdir -p /opt/ecommerce-lab",
      "mkdir -p /opt/ecommerce-db"
    ]
  }

  # Copy compiled Rust release binary directly from local build target into image
  provisioner "file" {
    source      = "../../../target/release/ecommerce_lab"
    destination = "/opt/ecommerce-lab/ecommerce_lab"
  }

  provisioner "shell" {
    inline = [
      "chmod +x /opt/ecommerce-lab/ecommerce_lab",
      "echo '✅ Golden Image Build & Rust Binary Integration Complete!'"
    ]
  }
}

