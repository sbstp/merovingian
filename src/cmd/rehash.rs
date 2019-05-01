use crate::config::Config;
use crate::error::Result;
use crate::io::fingerprint;
use crate::library::Library;

pub fn cmd_rehash(config: Config, library: &mut Library) -> Result {
    let root_path = config.root_path();

    for mut movie in library.all_movies()? {
        println!("Checking movie {}", movie.file.path);
        let fp = fingerprint::file(root_path.join(&movie.file.path))?;

        if fp != movie.file.fingerprint {
            println!("{:?} => {:?}", movie.file.fingerprint, fp);
            println!();
            movie.file.fingerprint = fp;
            library.save_file(&movie.file)?;
        }

        // for sub in movie.subtitles.iter_mut() {
        //     println!("Checking subtitle {}", sub.path.());
        //     let fp = fingerprint::file(root_path.join(&sub.path))?;

        //     if fp != sub.fingerprint {
        //         println!("{:?} => {:?}", sub.fingerprint, fp);
        //         println!();
        //         sub.fingerprint = fp;
        //     }
        // }
    }

    Ok(())
}
