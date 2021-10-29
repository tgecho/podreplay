CREATE TABLE feeds (
    id INTEGER NOT NULL PRIMARY KEY,
    uri TEXT NOT NULL,
    last_fetched DATETIME UTC,
    etag TEXT,
    UNIQUE(uri)
);
CREATE TABLE entries (
    id TEXT NOT NULL,
    feed_id INTEGER NOT NULL,
    noticed DATETIME UTC NOT NULL,
    published DATETIME UTC,
    PRIMARY KEY(id, noticed),
    UNIQUE(id, published),
    FOREIGN KEY (feed_id) REFERENCES feeds (id) ON DELETE RESTRICT ON UPDATE CASCADE
);
