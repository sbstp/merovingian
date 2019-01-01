use crate::mero::{Library, Result};
use crate::storage::Config;

pub fn cmd_sync(config: Config, library: &mut Library) -> Result {
    let root = config.root_path().to_path_buf();

    library.movies_mut().retain(|m| {
        let exists = root.join(&m.path).exists();
        if !exists {
            println!("Removing movie {}", m.path.display());
        }
        exists
    });

    for movie in library.movies_mut().iter_mut() {
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
