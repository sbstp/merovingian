use crate::mero::{Library, Result};
use crate::storage::Config;

pub fn cmd_sync(config: Config, library: &mut Library) -> Result {
    let root_path = config.root_path();

    library.movies_mut().retain(|m| {
        let exists = root_path.join(&m.path).exists();
        if !exists {
            println!("Removing movie {}", m.path.display());
        }
        exists
    });

    for movie in library.movies_mut().iter_mut() {
        movie.subtitles.retain(|s| {
            let exists = root_path.join(&s.path).exists();
            if !exists {
                println!("Removing subtitle {}", s.path.display());
            }
            exists
        });
    }

    library.commit()?;

    Ok(())
}
