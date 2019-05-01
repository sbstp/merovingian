use crate::config::Config;
use crate::error::Result;
use crate::library::Library;

pub fn cmd_sync(config: Config, library: &mut Library) -> Result {
    let root_path = config.root_path();

    for movie in library.all_movies()? {
        let exists = root_path.join(&movie.file.path).exists();
        if !exists {
            println!("Removing movie {}", movie.file.path);
            library.delete_movie(&movie)?;
        }
    }

    for mut movie in library.all_movies()? {
        library.load_subtitles(&mut movie)?;
        for subtitle in &movie.subtitles {
            let exists = root_path.join(&subtitle.file.path).exists();
            if !exists {
                println!("Removing subtitle {}", subtitle.file.path);
                library.delete_subtitle(&movie.id, subtitle)?;
            }
        }
    }

    Ok(())
}
