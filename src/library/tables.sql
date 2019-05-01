CREATE TABLE IF NOT EXISTS file (
    id BLOB NOT NULL,
    path TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_file_path ON file(path);
CREATE UNIQUE INDEX IF NOT EXISTS idx_file_fingerprint ON file(fingerprint);

CREATE TABLE IF NOT EXISTS movie (
    id BLOB NOT NULL,
    file_id BLOB NOT NULL,
    imdb_id INTEGER NOT NULL,
    primary_title TEXT NOT NULL,
    original_title TEXT NOT NULL,
    year INTEGER NOT NULL,
    FOREIGN KEY (file_id) REFERENCES file(id),
    PRIMARY KEY (id)
);

CREATE INDEX IF NOT EXISTS idx_movie_imdb_id ON movie(imdb_id);

CREATE TABLE IF NOT EXISTS subtitle (
    movie_id BLOB NOT NULL,
    file_id BLOB NOT NULL,
    lang TEXT NOT NULL,
    FOREIGN KEY (movie_id) REFERENCES movie(id),
    FOREIGN KEY (file_id) REFERENCES file(id),
    PRIMARY KEY (movie_id, file_id)
);

CREATE TABLE IF NOT EXISTS image (
    movie_id BLOB NOT NULL,
    file_id BLOB NOT NULL,
    kind TEXT NOT NULL,
    FOREIGN KEY (movie_id) REFERENCES movie(id),
    FOREIGN KEY (file_id) REFERENCES file(id),
    PRIMARY KEY (movie_id, file_id)
);
