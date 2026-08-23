# terraform/variables.tf

variable "hcloud_token" {
  type        = string
  description = "Hetzner Cloud API Token (Pass via export HCLOUD_TOKEN or terraform.tfvars)"
  sensitive   = true
  default     = ""
}

variable "location" {
  type        = string
  description = "Hetzner Datacenter location (fsn1 = Falkenstein Germany, nbg1 = Nuremberg, ash = Ashburn USA)"
  default     = "fsn1"
}

variable "server_type" {
  type        = string
  description = "Hetzner Cloud Server Type (cpx12 = 2 vCPU AMD EPYC, 2GB RAM)"
  default     = "cpx12"
}

variable "ssh_public_key_path" {
  type        = string
  description = "Path to local SSH public key"
  default     = "~/.ssh/id_ed25519.pub"
}

variable "ssh_private_key_path" {
  type        = string
  description = "Path to local SSH private key for Ansible connection"
  default     = "~/.ssh/id_ed25519"
}

variable "server_image" {
  type        = string
  description = "Hetzner Server Image or Golden Snapshot Name"
  default     = "ubuntu-24.04"
}

