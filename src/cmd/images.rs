use crate::error::Result;
use crate::io::fingerprint;
use crate::library::{self, Library};
use crate::scan::RelPath;
use crate::service::TMDB;
use crate::Config;

pub fn cmd_images(config: Config, library: &mut Library) -> Result {
    let root_path = config.root_path();

    let mut tmdb = TMDB::new(config.tmdb_cache_path());

    for mut movie in library.all_movies()? {
        if movie.images.len() == 0 {
            println!("Downloading images for {}", movie.file.path);

            if let Some(tmdb_title) = tmdb.find(movie.imdb_id)? {
                if let Some(poster_path) = &tmdb_title.poster_path {
                    let rel_path = RelPath::new(movie.file.path.with_file_name("poster.jpg")).unwrap();
                    let abs_path = root_path.join(&rel_path);

                    tmdb.get_save_image(&poster_path, &abs_path)?;

                    movie.images.push(library::Image::new(
                        library::File::new(rel_path, fingerprint::file(&abs_path)?),
                        "poster",
                    ));

                    println!("Poster downloaded");
                } else {
                    println!("Poster not found");
                }

                if let Some(backdrop_path) = &tmdb_title.backdrop_path {
                    let rel_path = RelPath::new(movie.file.path.with_file_name("backdrop.jpg")).unwrap();
                    let abs_path = root_path.join(&rel_path);

                    tmdb.get_save_image(&backdrop_path, root_path.join(root_path.join(&rel_path)))?;

                    movie.images.push(library::Image::new(
                        library::File::new(rel_path, fingerprint::file(&abs_path)?),
                        "backdrop",
                    ));

                    println!("Backdrop downloaded");
                } else {
                    println!("Backdrop not found");
                }

                library.save_movie(&movie)?;
            }
        }
    }

    Ok(())
}
