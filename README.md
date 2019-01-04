# Merovingian
Merovingian is a movie library manager, similar to `beets`. It can import movies from any location into a single, organized folder and rename them properly. It also supports management of subtitles and it can automatically detect the language of text-based subtitles. Files are copied using hard links when the import folder and the library folder are on the same device, making the import extremely fast and cheap. Files in the import folder can also be deleted without affecting the library folder.

Files are fingerprinted using a really fast method during the scan which means that files already in the library aren't imported over and over again. This means that a download folder, for instance, can be imported multiple times over time as new files are added and removed from it and only new movie files will be imported.

Matching and renaming is done using IMDB's publicly available dataset. It is downloaded automatically the first time it is needed (roughly 150MB in size). A small and efficient index is then produced with this dataset.

## Features
* Automatically detect movie files
* Match movie files with IMDB's title data
* Rename and hardlink or copy movie files
* Detect subtitle files, their language and rename them properly
* Maintain a library of existing movie and subtitles files
* File fingerprinting to avoid importing the same movie twice
* Detect duplicate movies using IMDB's title numbers
* TODO: automatically find movie posters and backdrops using themoviedb.org
* TODO: web UI/static site generator to browse movie collection
* TODO: query the library for titles, year ranges
* TODO: start a movie in the video player of your choice

## How it works:
1. Setup a folder where your movie files will be stored.
2. Import files from any location into that folder.

**The very first time you use mero, you need to setup the library folder where movies will be copied or hardlinked into. Use the `init` command.**

## Importing moves in your library
1. Run a scan on a directory that contains movies using the `scan` command.
2. The scan will contain the movie files' name and the match that was made with IMDB's title data, as well as a confidence score.
Use the `view` command to view the scan result and matches.
    * **Ignored** files are files that are already in your library and do not need to be imported.
    * **Unmatched** files are files whose title could not be found in the IMDB index.
    * **Duplicates** are files whose title is already in the database under a different version, i.e. two different copies of the same movie.
    * **Conflicts** are similar to duplicates, the difference being that neither of the files are in the library.
    * **Matches** are files that will be imported during an import since they aren't ignored, duplicates or conflicts. They are sorted by matching score, lowest first. So you should only have to pay attention to the first results, beyond a certain point all the matches should all be good.
3. Resolve any issues that that `view` command raised.
    * **Unmatched** files can be fixed by renaming the file to the correct title.
    * **Duplicates** can be fixed by either removing the file from the folder to be imported or by removing the other copy from the library and running the `sync` command. In that case the file in the folder to be imported will replace the file that was in the library.
    * **Conflicts** conflicts can be resolved by removing or ignoring all the files causing the conflicts but one.

    Make sure to run a new scan after renaming or removing files, and don't forget to run a `sync` command if you delete anything in the library folder.
4. Once you are satisfied with the status of your files, `apply` the scan to import the files into your library.


Example:
```bash
mero init ~/movies

# scan ~/downloads for movies
mero scan ~/downloads scan.mero

# view the scan result, ignored, unmatched, duplicates, conflicts and matches
mero view scan.mero | less

# if the scan is satisfactory, import
mero apply scan.mero
```

## License
AGPL 3.0, see `LICENSE`
