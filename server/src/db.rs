use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::{sqlite::SqliteRow, FromRow, Row};

#[derive(Debug)]
pub struct Feed {
    pub id: i64,
    pub uri: String,
    pub last_fetched: Option<DateTime<Utc>>,
    pub etag: Option<String>,
}

fn to_utc(ndt: NaiveDateTime) -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(ndt, Utc)
}

#[derive(Debug)]
pub struct Entry {
    pub id: i64,
    pub feed_id: i64,
    pub noticed: NaiveDateTime,
    pub published: Option<NaiveDateTime>,
}

#[cfg(test)]
#[test]
fn schemas() {
    let _ = sqlx::query!("SELECT * FROM feeds;").map(|r| Feed {
        id: r.id,
        uri: r.uri,
        last_fetched: r.last_fetched.map(to_utc),
        etag: r.etag,
    });
    let _ = sqlx::query_as!(Entry, "SELECT * FROM entries;");
}
