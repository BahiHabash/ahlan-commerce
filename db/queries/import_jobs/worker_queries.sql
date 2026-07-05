--! fetch_next_job(now) : (id, status, input_path, attempts, last_error?, created_at, updated_at)
UPDATE import_jobs
SET status = 'running',
    attempts = attempts + 1,
    updated_at = :now
WHERE id = (
    SELECT id 
    FROM import_jobs 
    WHERE status = 'queued' 
    ORDER BY created_at ASC 
    LIMIT 1 
    FOR UPDATE SKIP LOCKED
)
RETURNING *;

--! mark_job_succeeded(id, now)
UPDATE import_jobs
SET status = 'succeeded',
    updated_at = :now
WHERE id = :id AND status = 'running';

--! mark_job_failed(id, last_error, now)
UPDATE import_jobs
SET status = 'failed',
    last_error = :last_error,
    updated_at = :now
WHERE id = :id AND status = 'running';

--! requeue_job(id, now)
UPDATE import_jobs
SET status = 'queued',
    updated_at = :now
WHERE id = :id AND status = 'failed';
