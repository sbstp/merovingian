## Features
* Automatically detect movie files
* Match movie files with IMDB's title data
* Rename, hardlink or copy movie files
* Detect subtitle files, their language and rename them properly
* Maintain a library of existing movie and subtitles files
* File fingerprinting to avoid importing the same movie twice
* Detect duplicate movies using IMDB's title numbers
* TODO: automatically find movie posters and backdrops using themoviedb.org
* TODO: web UI/static site generator to browse movie collection

## How it works:

1. Run a scan on a directory that contains movies using the `scan` command.
2. The scan will contain the movie files' name and the match that was made with IMDB's title data, as well as a confidence score. Use the `view` command to view
the scan result and matches.
2. If the matches are satisfying, apply them with the `apply` command.
3. If the changes aren't satisfying, rename or delete files to improve the match and then run another scan. (i.e. fix typos, change the year).

Example:
```bash
flicks init ~/movies

# scan ~/downloads for movies
flicks scan ~/downloads scan.fls

# view the scan result, file names => title, year and imdb url
flicks view scan.fls | less

# if the scan is satisfactory, rename!
flicks apply scan.fls
```
