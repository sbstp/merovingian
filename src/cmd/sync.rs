use crate::flicks::{Library, Result};

pub fn cmd_sync(library: &mut Library) -> Result {
    let root = library.root().to_path_buf();

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
