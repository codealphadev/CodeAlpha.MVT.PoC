terraform {
  backend "gcs" {
    bucket = "codealpha-tf-state"
    prefix = "client-backend-logs"
  }
}
