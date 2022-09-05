resource "google_storage_bucket" "releases" {
  name                        = "codealpha-releases"
  location                    = local.region
  force_destroy               = true
  uniform_bucket_level_access = true
}


resource "google_service_account" "release_creation" {
  account_id   = "release-creation"
  display_name = "Release Creation"
  project      = google_project.project.project_id
}

resource "google_storage_bucket_iam_member" "storage_admin" {
  bucket = google_storage_bucket.releases.name
  role   = "roles/storage.admin"
  member = "serviceAccount:${google_service_account.release_creation.email}"
}


resource "google_storage_bucket_iam_member" "public_viewer" {
  bucket = google_storage_bucket.releases.name
  role   = "roles/storage.objectViewer"
  member = "allUsers"
}

module "gh_oidc" {
  source      = "terraform-google-modules/github-actions-runners/google//modules/gh-oidc"
  project_id  = google_project.project.project_id
  pool_id     = "codealpha-pool"
  provider_id = "codealpha-gh-provider"
  sa_mapping = {
    "release-service-account" = {
      sa_name   = "projects/${google_project.project.project_id}/serviceAccounts/${google_service_account.release_creation.email}"
      attribute = "attribute.repository/codealphadev/CodeAlpha.MVT.PoC"
    }
  }
}
resource "google_secret_manager_secret" "tauri_key_pw" {
  secret_id = "tauri_key_pw"
  replication { automatic = true }
}


resource "google_secret_manager_secret" "tauri_private_key" {
  secret_id = "tauri_private_key"
  replication { automatic = true }
}

output "provider_name" {
  value = module.gh_oidc.provider_name
}
