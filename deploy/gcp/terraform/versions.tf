terraform {
  required_version = ">= 1.5.0"

  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "~> 6.0"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.6"
    }
  }

  # Supply your own reviewed backend configuration; never reuse upstream state.
  backend "gcs" {}
}

provider "google" {
  project = var.project_id
  region  = var.region
}
