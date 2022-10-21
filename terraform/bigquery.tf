resource "google_bigquery_dataset" "dataset" {
  dataset_id                  = "active_users"
  friendly_name               = "Active users"
  location                    = local.region
}

resource "google_logging_project_sink" "default" {
  name = "bigquery-sink"
  destination = "bigquery.googleapis.com/projects/${google_project.project.project_id}/datasets/${google_bigquery_dataset.dataset.dataset_id}"
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

resource "google_bigquery_table" "internal_machine_id" {
  dataset_id = google_bigquery_dataset.dataset.dataset_id
  table_id   = "internal_machine_id"

  schema = <<EOF
[
    {
        "name": "machine_id",
        "type": "STRING",
        "mode": "REQUIRED"
    }
]
EOF

}

resource "google_bigquery_table" "default" {
    dataset_id = google_bigquery_dataset.dataset.dataset_id
    table_id = "active_users"

    view {
        use_legacy_sql = false
        query = <<EOT
        SELECT 
          DATE(timestamp) as dte, 
          COUNT(DISTINCT CASE WHEN labels.machine_id NOT IN (SELECT machine_id FROM `client-backend-x.active_users.internal_machine_id`) THEN labels.machine_id END) as foreign_users, 
          COUNT(DISTINCT labels.machine_id) as total_users 
          FROM `client-backend-x.active_users.client` 
          GROUP BY dte 
          ORDER by dte;
        EOT
    }
}

resource "google_bigquery_table" "feature_usage" {
    dataset_id = google_bigquery_dataset.dataset.dataset_id
    table_id = "feature_usage"

    view {
        use_legacy_sql = false
        query = <<EOT
        SELECT 
          DATE(timestamp) as dte,
          jsonPayload.feature as feature,
          COUNT(*) as count
          FROM `client-backend-x.active_users.client` 
          WHERE jsonPayload.metadata.level = "INFO"
          AND jsonPayload.feature IS NOT NULL
          AND labels.machine_id NOT IN (SELECT machine_id FROM `client-backend-x.active_users.internal_machine_id`)
          GROUP BY dte, feature 
          ORDER by dte;
        EOT
    }
}

