use crate::error::Result;
use crate::library::Library;

pub fn cmd_stats(library: &Library) -> Result {
    println!("There are {} movies in the library.", library.all_movies()?.len());

    Ok(())
}
