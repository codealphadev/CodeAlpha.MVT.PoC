locals {
  service         = "client-backend-x"
  product         = "codealpha"
  region          = "europe-west1"
  billing_account = "01EF6E-B7F8E7-7029D0"
  project         = local.service

  services_to_enable = [
    "logging.googleapis.com",
    "storage.googleapis.com",
    "iam.googleapis.com",
    "cloudresourcemanager.googleapis.com",
    "iamcredentials.googleapis.com",
    "sts.googleapis.com"
  ]
}
