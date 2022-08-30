locals {
  service         = "client-backend-logs"
  product         = "codealpha"
  region          = "europe-west1"
  billing_account = "01EF6E-B7F8E7-7029D0"
  project         = local.service

  services_to_enable = [
    "logging.googleapis.com",
  ]
}
