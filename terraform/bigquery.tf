resource "google_bigquery_dataset" "dataset" {
  dataset_id                  = "active_users"
  friendly_name               = "Active users"
  location                    = local.region
}

resource "google_logging_project_sink" "default" {
  name = "bigquery-sink"
  destination = "bigquery.googleapis.com/projects/${google_project.project.project_id}/datasets/${google_bigquery_dataset.dataset.dataset_id}"
  filter = "Checking for updates"
  unique_writer_identity = true

  bigquery_options {
    use_partitioned_tables = true
  }
}

resource "google_bigquery_dataset_iam_binding" "writer" {
  dataset_id = google_bigquery_dataset.dataset.dataset_id
  role       = "roles/bigquery.dataEditor"

  members = [
    google_logging_project_sink.default.writer_identity,
  ]
}

resource "google_bigquery_table" "default" {
    dataset_id = google_bigquery_dataset.dataset.dataset_id
    table_id = "active_users"

    view {
        use_legacy_sql = false
        query = <<EOT
        SELECT 
          DATE(timestamp) as dte, 
          COUNT(DISTINCT (CASE WHEN labels.machine_id != "CFC72E46-90FE-50BB-A3EF-B9C111A74831" AND labels.machine_id != "081C71C6-3AC8-5A6E-B503-B3330328E9A6" AND labels.machine_id != "7E3007D5-D32F-5163-8AD1-69D0557DCA36" AND labels.machine_id != "397E2E21-F462-56AB-9FE3-8B58A13AEFCE" THEN labels.machine_id END)) as foreign_users, 
          COUNT(DISTINCT labels.machine_id) as total_users 
          FROM `client-backend-x.active_users.client` 
          GROUP BY dte 
          ORDER by dte;
        EOT
    }
}
