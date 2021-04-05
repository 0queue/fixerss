CREATE TABLE items (
    id INTEGER PRIMARY KEY NOT NULL,
    channel_name VARCHAR NOT NULL,
    title VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    guid VARCHAR NOT NULL,
    pub_date DATETIME NOT NULL
);