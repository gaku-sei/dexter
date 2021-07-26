## Dexter

This small cli allows you to search for mangas hosted on MangaDex and read them.
The target is to become a full featured wrapper for the MangaDex website, and for the moment it already provides some valuable features.

This repository also contains a small `cbz-reader` which is not meant to be used as is for now (even though you can, it might not work with all CBZ files). It's used by `dexter` after you download a manga's chapter using the `--open` flag, see the `Usage` section for more.

### Usage

```
dexter 0.1.0

USAGE:
    dexter <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    chapters       Search for chapters
    download       Download and pack all the images contained in a chapter
    help           Prints this message or the help of the given subcommand(s)
    image-links    Display links to all the images contained in a chapter
    search         Search for mangas
```

### Example

Let's try to read the very first chapter of Detective Conan (because why not?).

It all starts with looking for the manga's id:

```bash
dexter search -t conan
```

Which returns a table like follow:

```
+-------------------------------------+--------------------------------------+
| Title                               | ID                                   |
+-------------------------------------+--------------------------------------+
| Detective Conan                     | 7f30dfc3-0b80-4dcc-a3b9-0cd746fac005 |
+-------------------------------------+--------------------------------------+
| ...                                 | ...                                  |
+-------------------------------------+--------------------------------------+
```

We can now use the returned ID to search for volumes and/or chapters, here volume 1, chapter 1:

```bash
dexter chapters -m 7f30dfc3-0b80-4dcc-a3b9-0cd746fac005 -v 1 -c 1
```

```
+-----------------------------------+--------------------------------------+--------+---------+----------+
| Title                             | ID                                   | Volume | Chapter | Language |
+-----------------------------------+--------------------------------------+--------+---------+----------+
| The Heisei Holmes                 | 07bf2a09-f30d-410f-aba1-025e2d27a88f | 1      | 1       | en       |
+-----------------------------------+--------------------------------------+--------+---------+----------+
| ...                               | ...                                  | ...    |...      | ...      |
+-----------------------------------+--------------------------------------+--------+---------+----------+
```

Using the chapter ID, we can now read it:

```bash
dexter download -c 07bf2a09-f30d-410f-aba1-025e2d27a88f -o
```

That'll automatically download the whole chapter as a CBZ file and open it in the simple `cbz-reader` which source is also available in this repository.
