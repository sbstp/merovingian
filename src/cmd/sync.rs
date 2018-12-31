use crate::mero::{Library, Result};
use crate::storage::Config;

pub fn cmd_sync(config: Config, library: &mut Library) -> Result {
    let root = config.root_path().to_path_buf();

    library.content.movies.retain(|m| {
        let exists = root.join(&m.path).exists();
        if !exists {
            println!("Removing movie {}", m.path.display());
        }
        exists
    });

    for movie in library.content.movies.iter_mut() {
        movie.subtitles.retain(|s| {
            let exists = root.join(&s.path).exists();
            if !exists {
                println!("Removing subtitle {}", s.path.display());
            }
            exists
        });
    }

    library.commit()?;

    Ok(())
}
