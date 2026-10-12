variable "project_id" {
  type        = string
  description = "GCP project. Shared tenancy: this module only creates edgequake-* plus VM elitizon-db."
}

variable "region" {
  type    = string
  default = "us-central1"
}

variable "zone" {
  type    = string
  default = "us-central1-a"
}

variable "name_prefix" {
  type        = string
  description = "GCP name prefix for bucket, secrets, WIF, SAs, firewalls, IP. VPC is PREFIX-host-vpc (leftover Option B already owns edgequake-vpc). Not the spec number."
  default     = "edgequake"
}

variable "instance_name" {
  type        = string
  description = "GCE VM name. Shared AGE+pgvector host (EdgeQuake today; other apps later)."
  default     = "tacitus-policy"
}

variable "machine_type" {
  type    = string
  default = "e2-standard-8"
}

variable "boot_disk_gb" {
  type    = number
  default = 100
}

variable "data_disk_gb" {
  type    = number
  default = 50
}

variable "edgequake_version" {
  type        = string
  description = "GHCR tag pin (LAW-148-4). Never latest."
}

variable "hostname" {
  type        = string
  description = "Public DNS name for Caddy Let's Encrypt. Empty = tls internal on the static IP."
  default     = ""
}

variable "caddy_acme_email" {
  type        = string
  description = "ACME contact when hostname is set."
  default     = ""
}

variable "github_repository" {
  type        = string
  description = "GitHub repo allowed to impersonate the deploy SA via WIF (org/name)."
  default     = "sargonxg/tacitus-political-policy-second-brain"
}

variable "operator_members" {
  type        = list(string)
  description = "IAM members granted IAP SSH + OS Login admin on this stack."
  default     = []
}

variable "enable_public_http_https" {
  type        = bool
  description = "Open :80 (redirect-only) and :443. Always true for Option A public host."
  default     = false
}
