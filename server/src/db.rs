use std::str::FromStr;
use std::{fmt::Debug, time::Duration};

use chrono::{DateTime, Utc};
use podreplay_lib::{CachedEntry, FeedMeta};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePool},
    ConnectOptions,
};
use tracing::log::LevelFilter;

#[derive(Clone)]
pub struct Db {
    uri: String,
    pool: SqlitePool,
}

impl Debug for Db {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Db").field("uri", &self.uri).finish()
    }
}

impl Db {
    pub async fn new(uri: String) -> Result<Self, sqlx::Error> {
        let default_prefix = "sqlite://";
        let uri = if !uri.starts_with(default_prefix) {
            default_prefix.to_string() + &uri
        } else {
            uri
        };

        let options = SqliteConnectOptions::from_str(&uri)?
            .log_statements(LevelFilter::Debug)
            .log_slow_statements(LevelFilter::Warn, Duration::from_millis(10));
        let pool = SqlitePool::connect_with(options).await?;
        let db = Db {
            uri: uri.clone(),
            pool,
        };
        tracing::info!("sqlite path: {}, version: {}", uri, db.get_version().await?);
        Ok(db)
    }

    pub async fn migrate(&self) -> Result<(), sqlx::migrate::MigrateError> {
        sqlx::migrate!("../migrations").run(&self.pool).await
    }

    pub async fn get_version(&self) -> Result<String, sqlx::Error> {
        sqlx::query_scalar!("SELECT sqlite_version()")
            .fetch_one(&self.pool)
            .await
            .and_then(|o| o.ok_or(sqlx::Error::RowNotFound))
    }

    #[tracing::instrument(level = "debug")]
    pub async fn update_feed_meta(
        &self,
        uri: &str,
        timestamp: &DateTime<Utc>,
        etag: &Option<String>,
    ) -> Result<FeedMeta, sqlx::Error> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO feeds (uri, first_fetched, last_fetched, etag)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(uri)
            DO UPDATE SET
                last_fetched=excluded.last_fetched,
                etag=excluded.etag
            RETURNING id
            ;"#,
            uri,
            timestamp,
            timestamp,
            etag
        )
        .fetch_one(&self.pool)
        .await?;
        sqlx::query_as!(FeedMeta, "SELECT * FROM feeds WHERE id = ?", id)
            .fetch_one(&self.pool)
            .await
    }

    #[tracing::instrument(level = "debug")]
    pub async fn get_entries(&self, feed_id: i64) -> Result<Vec<CachedEntry>, sqlx::Error> {
        sqlx::query_as!(
            CachedEntry,
            "SELECT * FROM entries WHERE feed_id = ? ORDER BY published, noticed, id",
            feed_id
        )
        .fetch_all(&self.pool)
        .await
    }

    #[tracing::instrument(level = "debug")]
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
