use crate::error::Result;
use crate::mero::Library;

pub fn cmd_stats(library: &Library) -> Result {
    println!("There are {} movies in the library.", library.movies().len());

    Ok(())
}
