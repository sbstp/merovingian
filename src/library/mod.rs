use std::path::Path;

use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::Result;
use crate::index::{Title, TitleId};
use crate::io::Fingerprint;
use crate::scan::RelPath;

pub struct File {
    pub id: Uuid,
    pub path: RelPath,
    pub fingerprint: Fingerprint,
}

impl File {
    pub fn new(path: RelPath, fingerprint: Fingerprint) -> File {
        File {
            id: Uuid::new_v4(),
            path,
            fingerprint,
        }
    }
}

pub struct Subtitle {
    pub file: File,
    pub lang: String,
}

impl Subtitle {
    pub fn new(file: File, lang: impl Into<String>) -> Subtitle {
        Subtitle {
            file,
            lang: lang.into(),
        }
    }
}

pub struct Image {
    pub file: File,
    pub kind: String,
}

impl Image {
    pub fn new(file: File, kind: impl Into<String>) -> Image {
        Image {
            file,
            kind: kind.into(),
        }
    }
}

pub struct Movie {
    pub id: Uuid,
    pub file: File,
    pub imdb_id: TitleId,
    pub primary_title: String,
    pub original_title: String,
    pub year: u16,
    pub subtitles: Vec<Subtitle>,
    pub images: Vec<Image>,
}

impl Movie {
    pub fn new(
        file: File,
        imdb_id: TitleId,
        primary_title: impl Into<String>,
        original_title: impl Into<String>,
        year: u16,
    ) -> Movie {
        Movie {
            id: Uuid::new_v4(),
            file,
            imdb_id,
            primary_title: primary_title.into(),
            original_title: original_title.into(),
            year,
            subtitles: vec![],
            images: vec![],
        }
    }
}

pub struct Library {
    con: Connection,
}

fn uuid_from_bytes(bytes: Vec<u8>) -> Uuid {
    uuid::Builder::from_slice(&bytes).expect("invalid blob").build()
}

impl Library {
    pub fn open(path: &Path) -> Result<Library> {
        let con = Connection::open(path)?;
        con.execute_batch(include_str!("tables.sql"))?;
        Ok(Library { con })
    }

    pub fn has_fingerprint(&self, fp: &Fingerprint) -> Result<bool> {
        let mut stmt = self.con.prepare("SELECT id FROM file WHERE fingerprint = ?")?;
        let exists = stmt.exists(params![fp.as_str()])?;
        Ok(exists)
    }

    pub fn has_title(&self, title: &Title) -> Result<bool> {
        let mut stmt = self.con.prepare("SELECT id FROM movie WHERE imdb_id = ?")?;
        let exists = stmt.exists(params![title.title_id.0])?;
        Ok(exists)
    }

    pub fn all_movies(&self) -> Result<Vec<Movie>> {
        let mut stmt = self.con.prepare(
            "SELECT m.id, m.imdb_id, m.primary_title, m.original_title, m.year, f.id, f.path, f.fingerprint
             FROM movie m
             INNER JOIN file f on f.id = m.file_id",
        )?;
        let mut rows = stmt.query(params![])?;

        let mut movies = vec![];

        while let Some(row) = rows.next()? {
            movies.push(Movie {
                id: uuid_from_bytes(row.get(0)?),
                imdb_id: TitleId::new(row.get(1)?),
                primary_title: row.get(2)?,
                original_title: row.get(3)?,
                year: row.get(4)?,
                file: File {
                    id: uuid_from_bytes(row.get(5)?),
                    path: RelPath::from_string(row.get(6)?),
                    fingerprint: Fingerprint::from_string(row.get(7)?),
                },
                subtitles: vec![],
                images: vec![],
            })
        }

        Ok(movies)
    }

    pub fn load_subtitles(&self, movie: &mut Movie) -> Result<()> {
        let mut stmt = self.con.prepare(
            "SELECT s.lang, f.id, f.path, f.fingerprint
             FROM subtitle s
             INNER JOIN file f on f.id = s.file_id
             WHERE s.movie_id = ?",
        )?;
        let mut rows = stmt.query(params![&movie.id.as_bytes()[..]])?;

        while let Some(row) = rows.next()? {
            movie.subtitles.push(Subtitle {
                lang: row.get(0)?,
                file: File {
                    id: uuid_from_bytes(row.get(1)?),
                    path: RelPath::from_string(row.get(2)?),
                    fingerprint: Fingerprint::from_string(row.get(3)?),
                },
            });
        }

        Ok(())
    }

    pub fn load_images(&self, movie: &mut Movie) -> Result<()> {
        let mut stmt = self.con.prepare(
            "SELECT i.kind, f.id, f.path, f.fingerprint
             FROM image i
             INNER JOIN file f on f.id = i.file_id
             WHERE i.movie_id = ?",
        )?;
        let mut rows = stmt.query(params![&movie.id.as_bytes()[..]])?;

        while let Some(row) = rows.next()? {
            movie.images.push(Image {
                kind: row.get(0)?,
                file: File {
                    id: uuid_from_bytes(row.get(1)?),
                    path: RelPath::from_string(row.get(2)?),
                    fingerprint: Fingerprint::from_string(row.get(3)?),
                },
            });
        }

        Ok(())
    }

    pub fn save_file(&self, file: &File) -> Result<()> {
        println!("saving file");
        println!("{}", file.path);
        self.con.execute(
            "INSERT INTO file (id, path, fingerprint) VALUES (?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET (path, fingerprint) = (excluded.path, excluded.fingerprint)",
            params![&file.id.as_bytes()[..], file.path.as_str(), file.fingerprint.as_str()],
        )?;
        Ok(())
    }

    pub fn save_subtitle(&self, movie_id: &Uuid, subtitle: &Subtitle) -> Result<()> {
        self.save_file(&subtitle.file)?;
        println!("saving sub");
        self.con.execute(
            "INSERT INTO subtitle (movie_id, file_id, lang) VALUES (?, ?, ?)
                 ON CONFLICT(movie_id, file_id) DO UPDATE SET lang = excluded.lang",
            params![
                &movie_id.as_bytes()[..],
                &subtitle.file.id.as_bytes()[..],
                subtitle.lang
            ],
        )?;
        Ok(())
    }

    pub fn save_image(&mut self, movie_id: &Uuid, image: &Image) -> Result<()> {
        self.save_file(&image.file)?;
        self.con.execute(
            "INSERT INTO image (movie_id, file_id, kind) VALUES (?, ?, ?)
                 ON CONFLICT (movie_id, file_id) DO UPDATE SET kind = excluded.kind",
            params![&movie_id.as_bytes()[..], &image.file.id.as_bytes()[..], image.kind],
        )?;
        Ok(())
    }

    pub fn save_movie(&mut self, movie: &Movie) -> Result<()> {
        println!("saving movie");
        self.save_file(&movie.file)?;
        self.con.execute(
            "INSERT INTO movie (id, file_id, imdb_id, primary_title, original_title, year) VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET (file_id, imdb_id, primary_title, original_title, year) =
               (excluded.file_id, excluded.imdb_id, excluded.primary_title, excluded.original_title, excluded.year)",
            params![
                &movie.id.as_bytes()[..],
                &movie.file.id.as_bytes()[..],
                movie.imdb_id.0,
                movie.primary_title,
                movie.original_title,
                movie.year
            ],
        )?;

        for subtitle in &movie.subtitles {
            self.save_subtitle(&movie.id, &subtitle)?;
        }

        for image in &movie.images {
            self.save_image(&movie.id, &image)?;
        }

        Ok(())
    }

    pub fn delete_file(&self, file: &File) -> Result<()> {
        self.con
            .execute("DELETE FROM file WHERE id = ?", params![&file.id.as_bytes()[..]])?;
        Ok(())
    }

    pub fn delete_subtitle(&mut self, movie_id: &Uuid, subtitle: &Subtitle) -> Result<()> {
        self.con.execute(
            "DELETE FROM subtitle WHERE movie_id = ? AND file_id = ?",
            params![&movie_id.as_bytes()[..], &subtitle.file.id.as_bytes()[..]],
        )?;
        self.delete_file(&subtitle.file)?;
        Ok(())
    }

    pub fn delete_image(&mut self, movie_id: &Uuid, image: &Image) -> Result<()> {
        self.con.execute(
            "DELETE FROM image WHERE movie_id = ? AND file_id = ?",
            params![&movie_id.as_bytes()[..], &image.file.id.as_bytes()[..]],
        )?;
        self.delete_file(&image.file)?;
        Ok(())
    }

    pub fn delete_movie(&mut self, movie: &Movie) -> Result<()> {
        for subtitle in &movie.subtitles {
            self.delete_subtitle(&movie.id, subtitle)?;
        }

        for image in &movie.images {
            self.delete_image(&movie.id, image)?;
        }

        self.con
            .execute("DELETE FROM movie WHERE id = ?", params![&movie.id.as_bytes()[..]])?;
        self.delete_file(&movie.file)?;
        Ok(())
    }
}
