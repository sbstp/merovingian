use crate::error::Result;
use crate::library::Library;

pub fn cmd_query(
    library: &Library,
    title: Option<String>,
    year: Option<u16>,
    mut year_gte: Option<u16>,
    mut year_lte: Option<u16>,
) -> Result {
    if let Some(year) = year {
        year_gte = Some(year);
        year_lte = Some(year);
    }

    let title = title.map(|t| t.to_lowercase());

    let mut movies = library.all_movies()?;
    movies.retain(|m| {
        if let Some(title) = &title {
            if !m.primary_title.to_lowercase().contains(&title[..])
                && !m.original_title.to_lowercase().contains(&title[..])
            {
                return false;
            }
        }
        if let Some(year) = year_gte {
            if m.year < year {
                return false;
            }
        }

        if let Some(year) = year_lte {
            if m.year > year {
                return false;
            }
        }

        true
    });

    movies.sort_by_key(|m| (m.year, m.primary_title.clone())); // TODO remove clone

    for m in &movies {
        println!("Primary title: {}", m.primary_title);
        println!("Year: {}", m.year);
        println!("URL: https://imdb.com/title/{}/", m.imdb_id.full());
        println!();
    }

    println!("{} results.", movies.len());

    Ok(())
}
