CREATE TABLE import_jobs (
  id UUID PRIMARY KEY,
  status TEXT NOT NULL CHECK (status IN ('queued', 'running', 'succeeded', 'failed')),
  input_path TEXT NOT NULL,
  attempts INTEGER NOT NULL CHECK (attempts >= 0 AND attempts <= 3),
  last_error TEXT,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL
);
