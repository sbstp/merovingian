use std::path::Path;

use crate::mero::{Index, Result};
use crate::storage::Scan;

pub fn cmd_view(path: impl AsRef<Path>, index: &Index) -> Result {
    let path = path.as_ref();

    let mut scan = Scan::load(path)?;

    println!("Ignored");
    println!("=======");
    for path in scan.ignored {
        println!("{}", path.display());
    }
    println!();

    println!("Unmatched");
    println!("=========");
    for path in scan.unmatched {
        println!("{}", path.display());
    }
    println!();

    println!("Duplicates");
    println!("==========");
    for path in scan.duplicates {
        println!("{}", path.display());
    }
    println!();

    scan.matches.sort_by_key(|m| m.score);

    println!("Matches");
    println!("=======");
    for mat in scan.matches {
        let title = &index.entries[&mat.title_id];
        println!("Path: {}", mat.path.display());
        println!("Name: {}", mat.path.file_name().and_then(|s| s.to_str()).unwrap());
        println!("Title: {}", title.primary_title);
        println!("Year: {}", title.year);
        println!("URL: https://imdb.com/title/tt{:07}/", mat.title_id);
        println!("Score: {:0.3}", mat.score);
        println!();
    }

    Ok(())
}
