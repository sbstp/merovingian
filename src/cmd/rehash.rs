use crate::mero::fingerprint;
use crate::mero::{Library, Result};

pub fn cmd_rehash(library: &mut Library) -> Result {
    for movie in library.movies_mut().iter_mut() {
        println!("Checking movie {}", movie.path.display());
        let fp = fingerprint::file(&movie.path)?;
        if fp != movie.fingerprint {
            println!("{:?} => {:?}", movie.fingerprint, fp);
            println!();
            movie.fingerprint = fp;
        }
        for sub in movie.subtitles.iter_mut() {
            println!("Checking subtitle {}", sub.path.display());
            let fp = fingerprint::file(&sub.path)?;
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
