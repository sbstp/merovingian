use crate::error::Result;
use crate::mero::{Library, Movie};

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

    let mut count = 0;

    let mut movies: Vec<&Movie> = library
        .movies()
        .iter()
        .filter(|m| {
            if let Some(title) = &title {
                if !m.identity.title.primary_title.to_lowercase().contains(&title[..]) {
                    return false;
                }
                if let Some(original_title) = &m.identity.title.original_title {
                    if !original_title.to_lowercase().contains(&title[..]) {
                        return false;
                    }
                }
            }
            if let Some(year) = year_gte {
                if m.identity.title.year < year {
                    return false;
                }
            }

            if let Some(year) = year_lte {
                if m.identity.title.year > year {
                    return false;
                }
            }

            true
        })
        .collect();

    movies.sort_by_key(|m| (m.identity.title.year, &m.identity.title.primary_title));

    for m in movies {
        count += 1;
        println!("Primary title: {}", m.identity.title.primary_title);
        println!("Year: {}", m.identity.title.year);
        println!("URL: https://imdb.com/title/{}/", m.identity.title.title_id.full());
        println!();
    }

    println!("{} results.", count);

    Ok(())
}
