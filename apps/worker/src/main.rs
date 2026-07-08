use catalog::{Catalog, RealClock, RealIdGenerator, CreateProductParams};
use db::queries::import_jobs::worker_queries::{fetch_next_job, mark_job_succeeded, mark_job_failed, requeue_job};
use deadpool_postgres::{Config as DbConfig, Runtime};
use tokio_postgres::NoTls;
use std::sync::Arc;
use serde::Deserialize;
use tokio::time::{sleep, Duration};

#[derive(Deserialize, Debug)]
struct ImportPayload {
    products: Vec<ImportProduct>,
}

#[derive(Deserialize, Debug)]
struct ImportProduct {
    title: String,
    handle: String,
    price_cents: u32,
    inventory_quantity: u32,
    published: bool,
    description: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "worker=info,catalog=info".into()),
        )
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let mut db_cfg = DbConfig::new();
    db_cfg.url = Some(database_url);
    let pool = db_cfg
        .create_pool(Some(Runtime::Tokio1), NoTls)
        .expect("Failed to create PostgreSQL connection pool");

    let clock = Arc::new(RealClock);
    let id_generator = Arc::new(RealIdGenerator);
    let catalog = Catalog::new(pool.clone(), clock.clone(), id_generator);

    tracing::info!("Ahlan Commerce Worker starting up");

    loop {
        match process_next_job(&pool, &catalog).await {
            Ok(true) => {
                // Job processed, check for next immediately
            }
            Ok(false) => {
                // No jobs, sleep
                sleep(Duration::from_secs(2)).await;
            }
            Err(e) => {
                tracing::error!("Error checking for jobs: {}", e);
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn process_next_job(pool: &deadpool_postgres::Pool, catalog: &Catalog) -> Result<bool, Box<dyn std::error::Error>> {
    let mut client = pool.get().await?;
    let now = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());

    // Fetch next job
    let job_opt = fetch_next_job()
        .bind(&client, &now)
        .opt()
        .await?;

    let job = match job_opt {
        Some(j) => j,
        None => return Ok(false), // No jobs
    };

    let job_id_str = job.id.to_string();
    tracing::info!(job_id = %job_id_str, attempt = job.attempts, status = "running", "Processing import job");

    // Read file
    let file_content_result: Result<String, _> = tokio::fs::read_to_string(&job.input_path).await;
    let file_content = match file_content_result {
        Ok(c) => c,
        Err(e) => {
            handle_job_failure(&mut client, &job.id, &format!("Failed to read file: {}", e), job.attempts, &now).await?;
            return Ok(true);
        }
    };

    // Parse JSON
    let payload: ImportPayload = match serde_json::from_str(&file_content) {
        Ok(p) => p,
        Err(e) => {
            handle_job_failure(&mut client, &job.id, &format!("Invalid JSON format: {}", e), job.attempts, &now).await?;
            return Ok(true);
        }
    };

    // Import products
    for product in payload.products {
        let params = CreateProductParams {
            title: product.title,
            handle: product.handle,
            price_cents: product.price_cents,
            inventory_quantity: product.inventory_quantity,
            published: product.published,
            description: product.description,
        };

        match catalog.create_product(params).await {
            Ok(_) => continue,
            Err(e) => {
                let err_msg = e.to_string();
                handle_job_failure(&mut client, &job.id, &err_msg, job.attempts, &now).await?;
                return Ok(true);
            }
        }
    }

    // Success
    let now = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    mark_job_succeeded()
        .bind(&client, &now, &job.id)
        .await?;

    tracing::info!(job_id = %job_id_str, attempt = job.attempts, status = "succeeded", "Import job completed successfully");
    Ok(true)
}

async fn handle_job_failure(
    client: &mut tokio_postgres::Client,
    job_id: &uuid::Uuid,
    error_msg: &str,
    attempts: i32,
    now: &chrono::DateTime<chrono::FixedOffset>,
) -> Result<(), Box<dyn std::error::Error>> {
    let job_id_str = job_id.to_string();
    tracing::error!(job_id = %job_id_str, attempt = attempts, status = "failed", error_code = %error_msg, "Import job failed");

    mark_job_failed()
        .bind(client, &error_msg, now, job_id)
        .await?;

    if attempts < 3 {
        tracing::info!(job_id = %job_id_str, attempt = attempts, status = "queued", "Requeuing failed import job");
        let now2 = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
        requeue_job()
            .bind(client, &now2, job_id)
            .await?;
    }

    Ok(())
}
