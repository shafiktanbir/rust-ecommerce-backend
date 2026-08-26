# terraform/main.tf
# Milestone V3 Multi-Node Hetzner Cloud Infrastructure (SRE Pattern)

# ─── 1. Access Pre-Registered SSH Key & Golden Image ─────────────────────────
data "hcloud_ssh_key" "default" {
  name = "shafikul@gophers-Latitude-3400"
}

data "hcloud_image" "golden_image" {
  with_selector = "project=rust-ecommerce-lab"
  most_recent   = true
}

# ─── 2. Private Network (VPC) ───────────────────────────────────────────────
resource "hcloud_network" "ecommerce_vpc" {
  name     = "ecommerce-vpc"
  ip_range = "10.0.0.0/16"
}

resource "hcloud_network_subnet" "api_subnet" {
  network_id   = hcloud_network.ecommerce_vpc.id
  type         = "cloud"
  network_zone = "eu-central"
  ip_range     = "10.0.1.0/24"
}

# ─── 3. Firewall Security Rules ─────────────────────────────────────────────
resource "hcloud_firewall" "cluster_fw" {
  name = "ecommerce-cluster-firewall"

  # Allow inbound SSH (port 22)
  rule {
    direction = "in"
    protocol  = "tcp"
    port      = "22"
    source_ips = [
      "0.0.0.0/0",
      "::/0"
    ]
  }

  # Allow inbound Nginx HTTP (port 8080)
  rule {
    direction = "in"
    protocol  = "tcp"
    port      = "8080"
    source_ips = [
      "0.0.0.0/0",
      "::/0"
    ]
  }

  # Allow internal private VPC traffic between nodes
  rule {
    direction = "in"
    protocol  = "tcp"
    port      = "any"
    source_ips = [
      "10.0.0.0/16"
    ]
  }

  rule {
    direction = "in"
    protocol  = "udp"
    port      = "any"
    source_ips = [
      "10.0.0.0/16"
    ]
  }
}

# ─── 4. Database & Redis Server (10.0.1.20) ─────────────────────────────────
resource "hcloud_server" "db_node" {
  name         = "ecommerce-db"
  image        = data.hcloud_image.golden_image.id
  server_type  = var.server_type
  location     = var.location
  ssh_keys     = [data.hcloud_ssh_key.default.id]
  firewall_ids = [hcloud_firewall.cluster_fw.id]

  public_net {
    ipv4_enabled = true
    ipv6_enabled = true
  }

  network {
    network_id = hcloud_network.ecommerce_vpc.id
    ip         = "10.0.1.20"
  }

  user_data = <<-EOF
    #!/bin/bash
    sysctl -w net.ipv4.conf.all.rp_filter=2 2>/dev/null || true
    sysctl -w net.ipv4.conf.default.rp_filter=2 2>/dev/null || true
    sysctl -w net.ipv4.conf.enp7s0.rp_filter=2 2>/dev/null || true

    cat << 'NET' > /etc/netplan/60-vpc.yaml
network:
  version: 2
  ethernets:
    enp7s0:
      dhcp4: true
NET
    chmod 600 /etc/netplan/60-vpc.yaml
    netplan apply

    mkdir -p /opt/ecommerce-db
    cat << 'DOCKER' > /opt/ecommerce-db/docker-compose.yml
    services:
      postgres:
        image: postgres:15-alpine
        container_name: ecommerce_lab_postgres
        environment:
          POSTGRES_USER: ecommerce
          POSTGRES_PASSWORD: ecommerce_secret
          POSTGRES_DB: ecommerce_lab
        ports:
          - "5432:5432"
        restart: unless-stopped

      redis:
        image: redis:7-alpine
        container_name: ecommerce_lab_redis
        ports:
          - "6379:6379"
        restart: unless-stopped
    DOCKER
    cd /opt/ecommerce-db && docker compose up -d
  EOF
}

# ─── 5. Axum API Worker 1 (10.0.1.11) ────────────────────────────────────────
resource "hcloud_server" "api1" {
  name         = "ecommerce-api-1"
  image        = data.hcloud_image.golden_image.id
  server_type  = var.server_type
  location     = var.location
  ssh_keys     = [data.hcloud_ssh_key.default.id]
  firewall_ids = [hcloud_firewall.cluster_fw.id]

  public_net {
    ipv4_enabled = true
    ipv6_enabled = true
  }

  network {
    network_id = hcloud_network.ecommerce_vpc.id
    ip         = "10.0.1.11"
  }

  user_data = <<-EOF
    #!/bin/bash
    sysctl -w net.ipv4.conf.all.rp_filter=2 2>/dev/null || true
    sysctl -w net.ipv4.conf.default.rp_filter=2 2>/dev/null || true
    sysctl -w net.ipv4.conf.enp7s0.rp_filter=2 2>/dev/null || true

    cat << 'NET' > /etc/netplan/60-vpc.yaml
network:
  version: 2
  ethernets:
    enp7s0:
      dhcp4: true
NET
    chmod 600 /etc/netplan/60-vpc.yaml
    netplan apply

    systemctl stop nginx || true
    systemctl disable nginx || true
    cat << 'SERVICE' > /etc/systemd/system/ecommerce-api.service
    [Unit]
    Description=Ecommerce Axum API Worker 1
    After=network.target

    [Service]
    Type=simple
    User=root
    WorkingDirectory=/opt/ecommerce-lab
    Environment="APP_PORT=8080"
    Environment="APP_ENV=production"
    Environment="DATABASE_URL=postgres://ecommerce:ecommerce_secret@10.0.1.20:5432/ecommerce_lab"
    Environment="REDIS_URL=redis://10.0.1.20:6379"
    Environment="RUST_LOG=ecommerce_lab=info,tower_http=warn"
    Environment="JWT_SECRET=supersecret_jwt_key_ecommerce_lab_2026"
    ExecStart=/opt/ecommerce-lab/ecommerce_lab
    Restart=always
    RestartSec=2s
    LimitNOFILE=65536

    [Install]
    WantedBy=multi-user.target
    SERVICE
    systemctl daemon-reload
    sleep 3
    systemctl enable --now ecommerce-api
  EOF
}

# ─── 6. Axum API Worker 2 (10.0.1.12) ────────────────────────────────────────
resource "hcloud_server" "api2" {
  name         = "ecommerce-api-2"
  image        = data.hcloud_image.golden_image.id
  server_type  = var.server_type
  location     = var.location
  ssh_keys     = [data.hcloud_ssh_key.default.id]
  firewall_ids = [hcloud_firewall.cluster_fw.id]

  public_net {
    ipv4_enabled = true
    ipv6_enabled = true
  }

  network {
    network_id = hcloud_network.ecommerce_vpc.id
    ip         = "10.0.1.12"
  }

  user_data = <<-EOF
    #!/bin/bash
    sysctl -w net.ipv4.conf.all.rp_filter=2 2>/dev/null || true
    sysctl -w net.ipv4.conf.default.rp_filter=2 2>/dev/null || true
    sysctl -w net.ipv4.conf.enp7s0.rp_filter=2 2>/dev/null || true

    cat << 'NET' > /etc/netplan/60-vpc.yaml
network:
  version: 2
  ethernets:
    enp7s0:
      dhcp4: true
NET
    chmod 600 /etc/netplan/60-vpc.yaml
    netplan apply

    systemctl stop nginx || true
    systemctl disable nginx || true
    cat << 'SERVICE' > /etc/systemd/system/ecommerce-api.service
    [Unit]
    Description=Ecommerce Axum API Worker 2
    After=network.target

    [Service]
    Type=simple
    User=root
    WorkingDirectory=/opt/ecommerce-lab
    Environment="APP_PORT=8080"
    Environment="APP_ENV=production"
    Environment="DATABASE_URL=postgres://ecommerce:ecommerce_secret@10.0.1.20:5432/ecommerce_lab"
    Environment="REDIS_URL=redis://10.0.1.20:6379"
    Environment="RUST_LOG=ecommerce_lab=info,tower_http=warn"
    Environment="JWT_SECRET=supersecret_jwt_key_ecommerce_lab_2026"
    ExecStart=/opt/ecommerce-lab/ecommerce_lab
    Restart=always
    RestartSec=2s
    LimitNOFILE=65536

    [Install]
    WantedBy=multi-user.target
    SERVICE
    systemctl daemon-reload
    sleep 3
    systemctl enable --now ecommerce-api
  EOF
}

# ─── 7. Nginx Load Balancer Server (10.0.1.10) ──────────────────────────────
resource "hcloud_server" "nginx_lb" {
  name         = "ecommerce-nginx-lb"
  image        = data.hcloud_image.golden_image.id
  server_type  = var.server_type
  location     = var.location
  ssh_keys     = [data.hcloud_ssh_key.default.id]
  firewall_ids = [hcloud_firewall.cluster_fw.id]

  public_net {
    ipv4_enabled = true
    ipv6_enabled = true
  }

  network {
    network_id = hcloud_network.ecommerce_vpc.id
    ip         = "10.0.1.10"
  }

  user_data = <<-EOF
    #!/bin/bash
    sysctl -w net.ipv4.conf.all.rp_filter=2 2>/dev/null || true
    sysctl -w net.ipv4.conf.default.rp_filter=2 2>/dev/null || true
    sysctl -w net.ipv4.conf.enp7s0.rp_filter=2 2>/dev/null || true

    cat << 'NET' > /etc/netplan/60-vpc.yaml
network:
  version: 2
  ethernets:
    enp7s0:
      dhcp4: true
NET
    chmod 600 /etc/netplan/60-vpc.yaml
    netplan apply

    cat << 'NGINX' > /etc/nginx/nginx.conf
    worker_processes auto;
    worker_rlimit_nofile 65536;

    events {
        worker_connections 10240;
        use epoll;
        multi_accept on;
    }

    http {
        access_log off;
        error_log /var/log/nginx/error.log warn;

        upstream ecommerce_cluster {
            least_conn;

            server 10.0.1.11:8080 max_fails=0;
            server 10.0.1.12:8080 max_fails=0;

            keepalive 512;
        }

        server {
            listen 8080 reuseport;
            server_name _;

            location / {
                proxy_pass http://ecommerce_cluster;
                proxy_http_version 1.1;
                proxy_set_header Connection "";
                proxy_set_header Host $host;
                proxy_set_header X-Real-IP $remote_addr;
                proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;

                proxy_buffering off;
            }
        }
    }
    NGINX
    systemctl restart nginx
  EOF
}
