use crate::error::Result;
use crate::library::Library;
use crate::service::TMDB;
use crate::Config;

pub fn cmd_images(config: Config, library: &mut Library) -> Result {
    let root_path = config.root_path();

    let tmdb = TMDB::new(config.tmdb_cache_path());

    // for key in library.movie_access_keys() {
    //     let mut movie = library.movie_mut(key);

    //     if movie.images.len() == 0 {
    //         println!("Downloading images for {}", movie.path.display());
    //         let tmdb_title = movie.identity.tmdb_title.clone();

    //         if let Some(poster_path) = &tmdb_title.poster_path {
    //             let rel_path = RelativePath::new(movie.path.with_file_name("poster.jpg"));
    //             tmdb.get_save_image(&poster_path, root_path.join(&rel_path))?;
    //             movie.images.push(rel_path);
    //             println!("Poster downloaded");
    //         } else {
    //             println!("Poster not found");
    //         }

    //         if let Some(backdrop_path) = &tmdb_title.backdrop_path {
    //             let rel_path = RelativePath::new(movie.path.with_file_name("backdrop.jpg"));
    //             tmdb.get_save_image(&backdrop_path, root_path.join(root_path.join(&rel_path)))?;
    //             movie.images.push(rel_path);
    //             println!("Backdrop downloaded");
    //         } else {
    //             println!("Backdrop not found");
    //         }

    //         drop(movie);
    //         library.commit()?;
    //     }
    // }

    // library.commit()?;

    Ok(())
}
