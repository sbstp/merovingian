use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};

use crate::error::Result;
use crate::index::Title;
use crate::io::Fingerprint;

pub struct File {
    id: u32,
    path: String, // TODO: RelativePath
    fingeprint: Fingerprint,
}

pub struct Subtitle {
    file: File,
    lang: String,
}

pub struct Image {
    file: File,
    kind: String,
}

pub struct Movie {
    id: u32,
    file: File,
    imdb_id: u32,
    primary_title: String,
    original_title: Option<String>,
    year: u32,
    subtitles: Vec<Subtitle>,
    images: Vec<Image>,
}

pub struct Library {
    con: Connection,
}

impl Library {
    pub fn open(path: &Path) -> Result<Library> {
        let con = Connection::open(path)?;
        con.execute_batch(include_str!("tables.sql"))?;
        Ok(Library { con })
    }

    pub fn has_fingerprint(&self, fp: &Fingerprint) -> Result<bool> {
        let stmt = self.con.prepare("SELECT id FROM file WHERE fingerprint = ?")?;
        let exists = stmt.exists(params![fp.as_str()])?;
        Ok(exists)
    }

    pub fn has_title(&self, title: &Title) -> Result<bool> {
        let stmt = self.con.prepare("SELECT id FROM movie WHERE imdb_id = ?")?;
        let exists = stmt.exists(params![title.title_id.0])?;
        Ok(exists)
    }

    pub fn all_movies(&self) -> Result<Vec<Movie>> {
        let stmt = self.con.prepare(
            "SELECT m.id, m.imdb_id, m.primary_title, m.original_title, m.year, f.id, f.path, f.fingerprint
             FROM movie m
             INNER JOIN file f on f.id = m.file_id",
        )?;
        let mut rows = stmt.query(params![])?;

        let mut movies = vec![];

        while let Some(row) = rows.next()? {
            movies.push(Movie {
                id: row.get(0)?,
                imdb_id: row.get(1)?,
                primary_title: row.get(2)?,
                original_title: row.get(3)?,
                year: row.get(4)?,
                file: File {
                    id: row.get(5)?,
                    path: row.get(6)?,
                    fingeprint: Fingerprint::from_string(row.get(7)?),
                },
                subtitles: vec![],
                images: vec![],
            })
        }

        Ok(movies)
    }

    pub fn save_file(&self, file: &File) -> Result<()> {
        self.con.execute(
            "INSERT INTO file (id, path, fingerprint) VALUES (?, ?, ?)
             ON CONFLICT(id) SET (path, fingerprint) = (excluded.path, excluded.fingerprint)",
            params![file.id, file.path, file.fingeprint.as_str()],
        )?;
        Ok(())
    }

    pub fn save_movie(&self, movie: &Movie) -> Result<()> {
        let tx = self.con.transaction()?;

        self.save_file(&movie.file)?;
        self.con.execute(
            "INSERT INTO movie (id, file_id, imdb_id, primary_title, original_title, year) VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) SET (file_id, imdb_id, primary_title, original_title, year) =
               (excluded.file_id, excluded.imdb_id, excluded.primary_title, excluded.original_title, excluded.year)",
            params![
                movie.id,
                movie.file.id,
                movie.imdb_id,
                movie.primary_title,
                movie.original_title,
                movie.year
            ],
        )?;

        for subtitle in &movie.subtitles {
            self.save_file(&subtitle.file);
            self.con.execute(
                "INSERT INTO subtitle (movie_id, file_id, lang) VALUES (?, ?, ?)
                 ON CONFLICT (movie_id, file_id) SET lang = excluded.lang",
                params![movie.id, subtitle.file.id, subtitle.lang],
            )?;
        }

        for image in &movie.images {
            self.save_file(&image.file);
            self.con.execute(
                "INSERT INTO image (movie_id, file_id, kind) VALUES (?, ?, ?)
                 ON CONFLICT (movie_id, file_id) SET kind = excluded.kind",
                params![movie.id, image.file.id, image.kind],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn load_subtitles(&self, movie: &mut Movie) -> Result<()> {
        let stmt = self.con.prepare(
            "SELECT s.lang, f.id, f.path, f.fingerprint
             FROM subtitle s
             INNER JOIN file f on f.id = s.file_id
             WHERE s.movie_id = ?",
        )?;
        let mut rows = stmt.query(params![movie.id])?;

        while let Some(row) = rows.next()? {
            movie.subtitles.push(Subtitle {
                lang: row.get(0)?,
                file: File {
                    id: row.get(1)?,
                    path: row.get(2)?,
                    fingeprint: Fingerprint::from_string(row.get(3)?),
                },
            });
        }

        Ok(())
    }

    pub fn load_images(&self, movie: &mut Movie) -> Result<()> {
        let stmt = self.con.prepare(
            "SELECT i.kind, f.id, f.path, f.fingerprint
             FROM image i
             INNER JOIN file f on f.id = i.file_id
             WHERE i.movie_id = ?",
        )?;
        let mut rows = stmt.query(params![movie.id])?;

        while let Some(row) = rows.next()? {
            movie.images.push(Image {
                kind: row.get(0)?,
                file: File {
                    id: row.get(1)?,
                    path: row.get(2)?,
                    fingeprint: Fingerprint::from_string(row.get(3)?),
                },
            });
        }

        Ok(())
    }
}
