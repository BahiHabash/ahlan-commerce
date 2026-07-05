// This file was generated with `cornucopia`. Do not modify.

#[derive(Clone, Copy, Debug)]
pub struct MarkJobSucceededParams {
    pub now: chrono::DateTime<chrono::FixedOffset>,
    pub id: uuid::Uuid,
}
#[derive(Debug)]
pub struct MarkJobFailedParams<T1: crate::StringSql> {
    pub last_error: T1,
    pub now: chrono::DateTime<chrono::FixedOffset>,
    pub id: uuid::Uuid,
}
#[derive(Clone, Copy, Debug)]
pub struct RequeueJobParams {
    pub now: chrono::DateTime<chrono::FixedOffset>,
    pub id: uuid::Uuid,
}
#[derive(Debug, Clone, PartialEq)]
pub struct FetchNextJob {
    pub id: uuid::Uuid,
    pub status: String,
    pub input_path: String,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}
pub struct FetchNextJobBorrowed<'a> {
    pub id: uuid::Uuid,
    pub status: &'a str,
    pub input_path: &'a str,
    pub attempts: i32,
    pub last_error: Option<&'a str>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}
impl<'a> From<FetchNextJobBorrowed<'a>> for FetchNextJob {
    fn from(
        FetchNextJobBorrowed {
            id,
            status,
            input_path,
            attempts,
            last_error,
            created_at,
            updated_at,
        }: FetchNextJobBorrowed<'a>,
    ) -> Self {
        Self {
            id,
            status: status.into(),
            input_path: input_path.into(),
            attempts,
            last_error: last_error.map(|v| v.into()),
            created_at,
            updated_at,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct FetchNextJobQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor: fn(&tokio_postgres::Row) -> Result<FetchNextJobBorrowed, tokio_postgres::Error>,
    mapper: fn(FetchNextJobBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> FetchNextJobQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(FetchNextJobBorrowed) -> R,
    ) -> FetchNextJobQuery<'c, 'a, 's, C, R, N> {
        FetchNextJobQuery {
            client: self.client,
            params: self.params,
            query: self.query,
            cached: self.cached,
            extractor: self.extractor,
            mapper,
        }
    }
    pub async fn one(self) -> Result<T, tokio_postgres::Error> {
        let row =
            crate::client::async_::one(self.client, self.query, &self.params, self.cached).await?;
        Ok((self.mapper)((self.extractor)(&row)?))
    }
    pub async fn all(self) -> Result<Vec<T>, tokio_postgres::Error> {
        self.iter().await?.try_collect().await
    }
    pub async fn opt(self) -> Result<Option<T>, tokio_postgres::Error> {
        let opt_row =
            crate::client::async_::opt(self.client, self.query, &self.params, self.cached).await?;
        Ok(opt_row
            .map(|row| {
                let extracted = (self.extractor)(&row)?;
                Ok((self.mapper)(extracted))
            })
            .transpose()?)
    }
    pub async fn iter(
        self,
    ) -> Result<
        impl futures::Stream<Item = Result<T, tokio_postgres::Error>> + 'c,
        tokio_postgres::Error,
    > {
        let stream = crate::client::async_::raw(
            self.client,
            self.query,
            crate::slice_iter(&self.params),
            self.cached,
        )
        .await?;
        let mapped = stream
            .map(move |res| {
                res.and_then(|row| {
                    let extracted = (self.extractor)(&row)?;
                    Ok((self.mapper)(extracted))
                })
            })
            .into_stream();
        Ok(mapped)
    }
}
pub struct FetchNextJobStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn fetch_next_job() -> FetchNextJobStmt {
    FetchNextJobStmt(
        "UPDATE import_jobs SET status = 'running', attempts = attempts + 1, updated_at = $1 WHERE id = ( SELECT id FROM import_jobs WHERE status = 'queued' ORDER BY created_at ASC LIMIT 1 FOR UPDATE SKIP LOCKED ) RETURNING *",
        None,
    )
}
impl FetchNextJobStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
        now: &'a chrono::DateTime<chrono::FixedOffset>,
    ) -> FetchNextJobQuery<'c, 'a, 's, C, FetchNextJob, 1> {
        FetchNextJobQuery {
            client,
            params: [now],
            query: self.0,
            cached: self.1.as_ref(),
            extractor:
                |row: &tokio_postgres::Row| -> Result<FetchNextJobBorrowed, tokio_postgres::Error> {
                    Ok(FetchNextJobBorrowed {
                        id: row.try_get(0)?,
                        status: row.try_get(1)?,
                        input_path: row.try_get(2)?,
                        attempts: row.try_get(3)?,
                        last_error: row.try_get(4)?,
                        created_at: row.try_get(5)?,
                        updated_at: row.try_get(6)?,
                    })
                },
            mapper: |it| FetchNextJob::from(it),
        }
    }
}
pub struct MarkJobSucceededStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn mark_job_succeeded() -> MarkJobSucceededStmt {
    MarkJobSucceededStmt(
        "UPDATE import_jobs SET status = 'succeeded', updated_at = $1 WHERE id = $2 AND status = 'running'",
        None,
    )
}
impl MarkJobSucceededStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
        now: &'a chrono::DateTime<chrono::FixedOffset>,
        id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[now, id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        MarkJobSucceededParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for MarkJobSucceededStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a MarkJobSucceededParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.now, &params.id))
    }
}
pub struct MarkJobFailedStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn mark_job_failed() -> MarkJobFailedStmt {
    MarkJobFailedStmt(
        "UPDATE import_jobs SET status = 'failed', last_error = $1, updated_at = $2 WHERE id = $3 AND status = 'running'",
        None,
    )
}
impl MarkJobFailedStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient, T1: crate::StringSql>(
        &'s self,
        client: &'c C,
        last_error: &'a T1,
        now: &'a chrono::DateTime<chrono::FixedOffset>,
        id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[last_error, now, id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync, T1: crate::StringSql>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        MarkJobFailedParams<T1>,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for MarkJobFailedStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a MarkJobFailedParams<T1>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.last_error, &params.now, &params.id))
    }
}
pub struct RequeueJobStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn requeue_job() -> RequeueJobStmt {
    RequeueJobStmt(
        "UPDATE import_jobs SET status = 'queued', updated_at = $1 WHERE id = $2 AND status = 'failed'",
        None,
    )
}
impl RequeueJobStmt {
    pub async fn prepare<'a, C: GenericClient>(
        mut self,
        client: &'a C,
    ) -> Result<Self, tokio_postgres::Error> {
        self.1 = Some(client.prepare(self.0).await?);
        Ok(self)
    }
    pub async fn bind<'c, 'a, 's, C: GenericClient>(
        &'s self,
        client: &'c C,
        now: &'a chrono::DateTime<chrono::FixedOffset>,
        id: &'a uuid::Uuid,
    ) -> Result<u64, tokio_postgres::Error> {
        client.execute(self.0, &[now, id]).await
    }
}
impl<'a, C: GenericClient + Send + Sync>
    crate::client::async_::Params<
        'a,
        'a,
        'a,
        RequeueJobParams,
        std::pin::Pin<
            Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
        >,
        C,
    > for RequeueJobStmt
{
    fn params(
        &'a self,
        client: &'a C,
        params: &'a RequeueJobParams,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<u64, tokio_postgres::Error>> + Send + 'a>,
    > {
        Box::pin(self.bind(client, &params.now, &params.id))
    }
}
