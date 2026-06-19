// This file was generated with `cornucopia`. Do not modify.

#[derive(Debug, Clone, PartialEq)]
pub struct ListPublishedProducts {
    pub id: String,
    pub title: String,
    pub handle: String,
    pub price_cents: i32,
    pub inventory_quantity: i32,
    pub published: bool,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub description: Option<String>,
    pub published_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}
pub struct ListPublishedProductsBorrowed<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub handle: &'a str,
    pub price_cents: i32,
    pub inventory_quantity: i32,
    pub published: bool,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub description: Option<&'a str>,
    pub published_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}
impl<'a> From<ListPublishedProductsBorrowed<'a>> for ListPublishedProducts {
    fn from(
        ListPublishedProductsBorrowed {
            id,
            title,
            handle,
            price_cents,
            inventory_quantity,
            published,
            created_at,
            updated_at,
            description,
            published_at,
        }: ListPublishedProductsBorrowed<'a>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            handle: handle.into(),
            price_cents,
            inventory_quantity,
            published,
            created_at,
            updated_at,
            description: description.map(|v| v.into()),
            published_at,
        }
    }
}
use crate::client::async_::GenericClient;
use futures::{self, StreamExt, TryStreamExt};
pub struct ListPublishedProductsQuery<'c, 'a, 's, C: GenericClient, T, const N: usize> {
    client: &'c C,
    params: [&'a (dyn postgres_types::ToSql + Sync); N],
    query: &'static str,
    cached: Option<&'s tokio_postgres::Statement>,
    extractor:
        fn(&tokio_postgres::Row) -> Result<ListPublishedProductsBorrowed, tokio_postgres::Error>,
    mapper: fn(ListPublishedProductsBorrowed) -> T,
}
impl<'c, 'a, 's, C, T: 'c, const N: usize> ListPublishedProductsQuery<'c, 'a, 's, C, T, N>
where
    C: GenericClient,
{
    pub fn map<R>(
        self,
        mapper: fn(ListPublishedProductsBorrowed) -> R,
    ) -> ListPublishedProductsQuery<'c, 'a, 's, C, R, N> {
        ListPublishedProductsQuery {
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
pub struct ListPublishedProductsStmt(&'static str, Option<tokio_postgres::Statement>);
pub fn list_published_products() -> ListPublishedProductsStmt {
    ListPublishedProductsStmt(
        "SELECT * FROM products WHERE published = true ORDER BY created_at DESC",
        None,
    )
}
impl ListPublishedProductsStmt {
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
    ) -> ListPublishedProductsQuery<'c, 'a, 's, C, ListPublishedProducts, 0> {
        ListPublishedProductsQuery {
            client,
            params: [],
            query: self.0,
            cached: self.1.as_ref(),
            extractor: |
                row: &tokio_postgres::Row,
            | -> Result<ListPublishedProductsBorrowed, tokio_postgres::Error> {
                Ok(ListPublishedProductsBorrowed {
                    id: row.try_get(0)?,
                    title: row.try_get(1)?,
                    handle: row.try_get(2)?,
                    price_cents: row.try_get(3)?,
                    inventory_quantity: row.try_get(4)?,
                    published: row.try_get(5)?,
                    created_at: row.try_get(6)?,
                    updated_at: row.try_get(7)?,
                    description: row.try_get(8)?,
                    published_at: row.try_get(9)?,
                })
            },
            mapper: |it| ListPublishedProducts::from(it),
        }
    }
}
