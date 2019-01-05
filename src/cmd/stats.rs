use crate::mero::{Library, Result};

pub fn cmd_stats(library: &Library) -> Result {
    println!("There are {} movies in the library.", library.movies().len());

    Ok(())
}
