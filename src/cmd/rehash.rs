use crate::mero::fingerprint;
use crate::mero::{Library, Result};
use crate::storage::Config;

pub fn cmd_rehash(config: Config, library: &mut Library) -> Result {
    let root_path = config.root_path();

    for movie in library.movies_mut().iter_mut() {
        println!("Checking movie {}", movie.path.display());
        let fp = fingerprint::file(root_path.join(&movie.path))?;

        if fp != movie.fingerprint {
            println!("{:?} => {:?}", movie.fingerprint, fp);
            println!();
            movie.fingerprint = fp;
        }
        for sub in movie.subtitles.iter_mut() {
            println!("Checking subtitle {}", sub.path.display());
            let fp = fingerprint::file(root_path.join(&sub.path))?;

            if fp != sub.fingerprint {
                println!("{:?} => {:?}", sub.fingerprint, fp);
                println!();
                sub.fingerprint = fp;
            }
        }
    }

    library.commit()?;

    Ok(())
}
