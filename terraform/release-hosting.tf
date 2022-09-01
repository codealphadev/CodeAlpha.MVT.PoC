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
  member = "serviceAccount:${google_service_account.tracing.email}"
}


resource "google_storage_bucket_iam_member" "public_viewer" {
  bucket = google_storage_bucket.releases.name
  role   = "roles/storage.objectViewer"
  member = "allUsers"
}

