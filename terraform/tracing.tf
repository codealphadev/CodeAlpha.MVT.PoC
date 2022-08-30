resource "google_service_account" "tracing" {
  account_id   = "client-tracing"
  display_name = "Client Tracing"
  project      = google_project.project.project_id
}

resource "google_project_iam_binding" "tracing" {
  project = google_project.project.project_id
  role    = "roles/logging.logWriter"
  members = [
    "serviceAccount:${google_service_account.tracing.email}"
  ]
}
