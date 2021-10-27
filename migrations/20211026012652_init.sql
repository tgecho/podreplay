CREATE TABLE feeds (
    id INTEGER NOT NULL PRIMARY KEY,
    uri TEXT NOT NULL,
    last_fetched DATETIME,
    etag TEXT
);
CREATE TABLE entries (
    id INTEGER NOT NULL,
    feed_id INTEGER NOT NULL,
    noticed DATETIME NOT NULL,
    published DATETIME,
    PRIMARY KEY(id, noticed),
    UNIQUE(id, published),
    FOREIGN KEY (feed_id) REFERENCES feeds (id) ON DELETE RESTRICT ON UPDATE CASCADE
);
