# terraform/main.tf
# Milestone V3 Multi-Node Hetzner Cloud Infrastructure

# ─── 1. Access Pre-Registered SSH Key ─────────────────────────────────────────
data "hcloud_ssh_key" "default" {
  name = "shafikul@gophers-Latitude-3400"
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
  name        = "ecommerce-db"
  image       = var.server_image
  server_type = var.server_type
  location    = var.location
  ssh_keys    = [data.hcloud_ssh_key.default.id]
  firewall_ids = [hcloud_firewall.cluster_fw.id]
  user_data   = "#!/bin/bash\necho 'Server initialized'"
}

resource "hcloud_server_network" "db_vpc" {
  server_id  = hcloud_server.db_node.id
  network_id = hcloud_network.ecommerce_vpc.id
  ip         = "10.0.1.20"
}

# ─── 5. Axum API Worker 1 (10.0.1.11) ────────────────────────────────────────
resource "hcloud_server" "api1" {
  name        = "ecommerce-api-1"
  image       = var.server_image
  server_type = var.server_type
  location    = var.location
  ssh_keys    = [data.hcloud_ssh_key.default.id]
  firewall_ids = [hcloud_firewall.cluster_fw.id]
  user_data   = "#!/bin/bash\necho 'Server initialized'"
}

resource "hcloud_server_network" "api1_vpc" {
  server_id  = hcloud_server.api1.id
  network_id = hcloud_network.ecommerce_vpc.id
  ip         = "10.0.1.11"
}

# ─── 6. Axum API Worker 2 (10.0.1.12) ────────────────────────────────────────
resource "hcloud_server" "api2" {
  name        = "ecommerce-api-2"
  image       = var.server_image
  server_type = var.server_type
  location    = var.location
  ssh_keys    = [data.hcloud_ssh_key.default.id]
  firewall_ids = [hcloud_firewall.cluster_fw.id]
  user_data   = "#!/bin/bash\necho 'Server initialized'"
}

resource "hcloud_server_network" "api2_vpc" {
  server_id  = hcloud_server.api2.id
  network_id = hcloud_network.ecommerce_vpc.id
  ip         = "10.0.1.12"
}

# ─── 7. Nginx Load Balancer Server (10.0.1.10) ──────────────────────────────
resource "hcloud_server" "nginx_lb" {
  name        = "ecommerce-nginx-lb"
  image       = var.server_image
  server_type = var.server_type
  location    = var.location
  ssh_keys    = [data.hcloud_ssh_key.default.id]
  firewall_ids = [hcloud_firewall.cluster_fw.id]
  user_data   = "#!/bin/bash\necho 'Server initialized'"
}

resource "hcloud_server_network" "nginx_vpc" {
  server_id  = hcloud_server.nginx_lb.id
  network_id = hcloud_network.ecommerce_vpc.id
  ip         = "10.0.1.10"
}
