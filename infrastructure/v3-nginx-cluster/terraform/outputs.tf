# terraform/outputs.tf

output "nginx_lb_public_ip" {
  description = "Public IP address of the Nginx Load Balancer (Target for k6 load test)"
  value       = hcloud_server.nginx_lb.ipv4_address
}

output "api_1_private_ip" {
  description = "Private IP address of API Worker 1"
  value       = "10.0.1.11"
}

output "api_2_private_ip" {
  description = "Private IP address of API Worker 2"
  value       = "10.0.1.12"
}

output "db_private_ip" {
  description = "Private IP address of Postgres + Redis Database Node"
  value       = "10.0.1.20"
}

# ─── Auto-Generate Ansible Inventory File ────────────────────────────────────
resource "local_file" "ansible_inventory" {
  filename = "${path.module}/../ansible/hosts.ini"
  content  = <<-EOF
[nginx_lb]
${hcloud_server.nginx_lb.ipv4_address} ansible_user=root private_ip=10.0.1.10

[api_workers]
10.0.1.11 app_port=8080 worker_name=api1
10.0.1.12 app_port=8080 worker_name=api2

[db_node]
10.0.1.20

[all:vars]
ansible_ssh_private_key_file=${var.ssh_private_key_path}
ansible_ssh_common_args='-o StrictHostKeyChecking=no'
  EOF
}
