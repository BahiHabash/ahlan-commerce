# Deployment With Coolify

This runbook documents the service topology, environment variables, and deployment procedures for running Ahlan Commerce in Coolify.

## Service Topology

The deployment consists of the following independent services deployed within Coolify:

1. **PostgreSQL Database**
   - Service Type: Database
   - Description: The primary relational database.
   - Internal URL provided by Coolify to other services.

2. **Redis Cache**
   - Service Type: Service (Redis)
   - Description: Task queuing and caching backend.
   - Internal URL provided by Coolify to other services.

3. **API Service (Rust)**
   - Service Type: Application (Docker based)
   - Build Command: `make build-release`
   - Start Command: `api`
   - Pre-Deploy Script: `atlas migrate apply --env production --url $DATABASE_URL`
   - Port Expose: `3000`
   - Port Mapping: `8081:3000`

4. **Worker Service (Rust)**
   - Service Type: Application (Docker based)
   - Build Command: `make build-release`
   - Start Command: `worker`

5. **Admin Frontend (Next.js/React/Vite)**
   - Service Type: Application (Docker based)
   - Dockerfile: `/Dockerfile.admin`
   - Start Command: `npm start`
   - Port Expose: `3000`
   - Port Mapping: `8080:3000`

## Environment Variables

The following environment variables must be configured in your Coolify instances:

### API Service
- `DATABASE_URL`: Full PostgreSQL connection string (provided by Coolify DB service).
- `REDIS_URL`: Full Redis connection string (provided by Coolify Redis service).
- `API_BIND_ADDR`: `0.0.0.0:3000`

### Worker Service
- `DATABASE_URL`: Full PostgreSQL connection string.
- `REDIS_URL`: Full Redis connection string.

### Admin Frontend
- `ADMIN_PUBLIC_API_URL`: `http://178.104.61.10:8081`

## Important URLs

Once deployed, you can verify the deployment at the following URLs:

- **Public App URL**: `http://178.104.61.10:8080`
- **API Health Endpoint**: `http://178.104.61.10:8081/health`
- **Generated Docs**: `http://178.104.61.10:8081/docs` (if enabled in production)

## Atlas Migrations (Manual Fallback in CI)

If the database migration check fails in CI due to missing service components, the manual fallback command to verify migrations against a local test database is:

```bash
make db-start
make db-create
make db-migrate
```

## Troubleshooting & Debugging

- **Health Checks Failing**: Ensure the environment variables (`DATABASE_URL`, `REDIS_URL`) are correct. The API and Worker are designed to fail loudly at startup if env vars are missing.
- **Worker Not Processing Jobs**: Check the Worker service logs in the Coolify dashboard. Verify it has the correct `REDIS_URL` and `DATABASE_URL`.
- **Database Migrations Issue**: Check the API Service deploy logs. The `atlas migrate apply` step runs in the pre-deploy script. If it fails, the new deployment is aborted.
- **Rollback**: If a deployment fails or introduces a critical bug, use the Coolify dashboard to rollback to the previous successful deployment. The rollback button is available in the deployment history of the application.
- **Redeploy**: You can trigger a redeploy from the Coolify dashboard or by pushing a new commit to the repository, which Coolify can pick up automatically if Git hooks are configured.
