# terraform/outputs.tf

output "nginx_lb_public_ip" {
  description = "Public IP address of the Nginx Load Balancer (Target for k6 load test)"
  value       = hcloud_server.nginx_lb.ipv4_address
}

output "api_1_public_ip" {
  description = "Public IP address of API Worker 1"
  value       = hcloud_server.api1.ipv4_address
}

output "api_2_public_ip" {
  description = "Public IP address of API Worker 2"
  value       = hcloud_server.api2.ipv4_address
}

output "db_public_ip" {
  description = "Public IP address of Postgres + Redis Database Node"
  value       = hcloud_server.db_node.ipv4_address
}

# ─── Auto-Generate Ansible Inventory File ────────────────────────────────────
resource "local_file" "ansible_inventory" {
  filename = "${path.module}/../ansible/hosts.ini"
  content  = <<-EOF
[nginx_lb]
${hcloud_server.nginx_lb.ipv4_address} ansible_user=root private_ip=10.0.1.10

[api_workers]
${hcloud_server.api1.ipv4_address} ansible_user=root private_ip=10.0.1.11 app_port=8081 worker_name=api1
${hcloud_server.api2.ipv4_address} ansible_user=root private_ip=10.0.1.12 app_port=8082 worker_name=api2

[db_node]
${hcloud_server.db_node.ipv4_address} ansible_user=root private_ip=10.0.1.20

[all:vars]
ansible_ssh_private_key_file=${var.ssh_private_key_path}
ansible_ssh_common_args='-o StrictHostKeyChecking=no'
  EOF
}
