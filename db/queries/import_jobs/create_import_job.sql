--! create_import_job(id, status, input_path, attempts, last_error?, created_at, updated_at) : (id, status, input_path, attempts, last_error?, created_at, updated_at)
INSERT INTO import_jobs (id, status, input_path, attempts, last_error, created_at, updated_at)
VALUES (:id, :status, :input_path, :attempts, :last_error, :created_at, :updated_at)
RETURNING *;
