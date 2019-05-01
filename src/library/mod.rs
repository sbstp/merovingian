mod sql_builder;

use std::path::Path;

use log::debug;
use rusqlite::{named_params, params, Connection};
use uuid::Uuid;

use self::sql_builder::insert_into;
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
                id: row.get(0)?,
                imdb_id: TitleId::new(row.get(1)?),
                primary_title: row.get(2)?,
                original_title: row.get(3)?,
                year: row.get(4)?,
                file: File {
                    id: row.get(5)?,
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
                    id: row.get(1)?,
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
                    id: row.get(1)?,
                    path: RelPath::from_string(row.get(2)?),
                    fingerprint: Fingerprint::from_string(row.get(3)?),
                },
            });
        }

        Ok(())
    }

    pub fn save_file(&self, file: &File) -> Result<()> {
        debug!("saving file path={}", file.path);

        self.con.execute_named(
            &insert_into("file", &["id", "path", "fingerprint"])
                .on_conflict_update(&["id"])
                .to_string(),
            named_params! {
                ":id": file.id,
                ":path": file.path.as_str(),
                ":fingerprint": file.fingerprint.as_str(),
            },
        )?;

        Ok(())
    }

    pub fn save_subtitle(&self, movie_id: &Uuid, subtitle: &Subtitle) -> Result<()> {
        debug!("saving subtitle lang={}", subtitle.lang);

        self.save_file(&subtitle.file)?;

        self.con.execute_named(
            &insert_into("subtitle", &["movie_id", "file_id", "lang"])
                .on_conflict_update(&["movie_id", "file_id"])
                .to_string(),
            named_params! {
                ":movie_id": movie_id,
                ":file_id": subtitle.file.id,
                ":lang": subtitle.lang,
            },
        )?;

        Ok(())
    }

    pub fn save_image(&mut self, movie_id: &Uuid, image: &Image) -> Result<()> {
        debug!("saving image kind={}", image.kind);

        self.save_file(&image.file)?;

        self.con.execute_named(
            &insert_into("image", &["movie_id", "file_id", "kind"])
                .on_conflict_update(&["movie_id", "file_id"])
                .to_string(),
            named_params! {
                ":movie_id": movie_id,
                ":file_id": image.file.id,
                ":kind": image.kind,
            },
        )?;

        Ok(())
    }

    pub fn save_movie(&mut self, movie: &Movie) -> Result<()> {
        debug!("saving movie title={}", movie.primary_title);

        self.save_file(&movie.file)?;

        self.con.execute_named(
            &insert_into(
                "movie",
                &["id", "file_id", "imdb_id", "primary_title", "original_title", "year"],
            )
            .on_conflict_update(&["id"])
            .to_string(),
            named_params! {
                ":id": movie.id,
                ":file_id": movie.file.id,
                ":imdb_id": movie.imdb_id.0,
                ":primary_title": movie.primary_title,
                ":original_title": movie.original_title,
                ":year": movie.year,
            },
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
        debug!("deleting file path={}", file.path);

        self.con.execute("DELETE FROM file WHERE id = ?", params![file.id])?;
        Ok(())
    }

    pub fn delete_subtitle(&mut self, movie_id: &Uuid, subtitle: &Subtitle) -> Result<()> {
        debug!("deleting subtitle lang={}", subtitle.lang);

        self.con.execute(
            "DELETE FROM subtitle WHERE movie_id = ? AND file_id = ?",
            params![movie_id, subtitle.file.id],
        )?;
        self.delete_file(&subtitle.file)?;
        Ok(())
    }

    pub fn delete_image(&mut self, movie_id: &Uuid, image: &Image) -> Result<()> {
        debug!("deleting image kind={}", image.kind);

        self.con.execute(
            "DELETE FROM image WHERE movie_id = ? AND file_id = ?",
            params![movie_id, image.file.id],
        )?;
        self.delete_file(&image.file)?;
        Ok(())
    }

    pub fn delete_movie(&mut self, movie: &Movie) -> Result<()> {
        debug!("deleting movie title={}", movie.primary_title);

        for subtitle in &movie.subtitles {
            self.delete_subtitle(&movie.id, subtitle)?;
        }

        for image in &movie.images {
            self.delete_image(&movie.id, image)?;
        }

        self.con.execute("DELETE FROM movie WHERE id = ?", params![movie.id])?;
        self.delete_file(&movie.file)?;
        Ok(())
    }
}
