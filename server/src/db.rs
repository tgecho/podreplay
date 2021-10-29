use chrono::{DateTime, Utc};
use podreplay_lib::{CachedEntry, FeedMeta};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub async fn new(uri: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(uri).await?;
        Ok(Db { pool })
    }

    pub async fn get_version(&self) -> Result<String, sqlx::Error> {
        sqlx::query_scalar!("SELECT sqlite_version();")
            .fetch_one(&self.pool)
            .await
            .map(|o| o.unwrap())
    }

    pub async fn get_feed_meta(&self, feed_uri: &str) -> Result<Option<FeedMeta>, sqlx::Error> {
        sqlx::query_as!(FeedMeta, "SELECT * FROM feeds WHERE uri = ?", feed_uri)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn update_feed_meta(
        &self,
        uri: &str,
        last_fetched: &DateTime<Utc>,
        etag: Option<&str>,
    ) -> Result<i64, sqlx::Error> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO feeds (uri, last_fetched, etag)
            VALUES (?, ?, ?)
            ON CONFLICT(uri)
            DO UPDATE SET
                last_fetched=excluded.last_fetched,
                etag=excluded.etag
            RETURNING id
            ;"#,
            uri,
            last_fetched,
            etag
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap(); // TODO: handle this?
        Ok(id)
    }

    pub async fn get_entries(&self, feed_id: i64) -> Result<Vec<CachedEntry>, sqlx::Error> {
        sqlx::query_as!(
            CachedEntry,
            "SELECT * FROM entries WHERE feed_id = ? ORDER BY published, noticed, id",
            feed_id
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn update_cached_entries(
        &self,
        feed_id: i64,
        entries: &[CachedEntry],
    ) -> Result<Vec<CachedEntry>, sqlx::Error> {
        // TODO: make this a proper bulk insert and figure out a way to maybe
        // use RETURNING to avoid this whole "refetch everything" approach.
        // Simplicity for now!
        for entry in entries {
            sqlx::query!(
                r#"
                INSERT INTO entries (id, feed_id, noticed, published)
                VALUES (?, ?, ?, ?)
                ON CONFLICT DO NOTHING
                ;"#,
                entry.id,
                entry.feed_id,
                entry.noticed,
                entry.published
            )
            .execute(&self.pool)
            .await?;
        }
        self.get_entries(feed_id).await
    }
}

#[cfg(test)]
#[test]
fn schemas() {
    let _ = sqlx::query_as!(FeedMeta, "SELECT * FROM feeds;");
    let _ = sqlx::query_as!(CachedEntry, "SELECT * FROM entries;");
}
