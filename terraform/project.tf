data "google_folder" "client_backend" {
  folder = "126328635215"
}

resource "google_project" "project" {
  billing_account = local.billing_account
  name            = local.project
  project_id      = local.project
  folder_id       = data.google_folder.client_backend.folder_id
}

resource "google_project_service" "project" {
  project  = local.project
  for_each = toset(local.services_to_enable)
  service  = each.value

  disable_dependent_services = true
}
