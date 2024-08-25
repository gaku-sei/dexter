# Dexter (cli) and Sinister (gui) wrappers for MangaDex with some QOL features

Tools:

- `dexter` - cli - A simple MangaDex client
- `sinister` - gui - _in progress_ - A gui client for MangaDex

## Dexter

This small cli allows you to search for mangas hosted on MangaDex and read them.
The target is to become a full featured wrapper for the MangaDex website, and for the moment it already provides some valuable features.

### Usage

```
Usage: dexter.exe <COMMAND>

Commands:
  interactive-search  Interactive Search
  search              Search for mangas
  chapters            Search for chapters
  image-links         Display links to all the images contained in a chapter
  download            Download and pack all the images contained in a chapter
  help                Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### Example

Let's read the very first chapter of Detective Conan.

It all starts with looking for the manga's id:

```bash
dexter search -t conan
```

Which returns a table as follows:

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

Using the chapter ID, we can now download it:

```bash
dexter download -c 07bf2a09-f30d-410f-aba1-025e2d27a88f -o
```

That'll automatically download the whole chapter as a CBZ file and open it using the simple [`eco-view`](https://github.com/gaku-sei/eco/tree/main/eco-view).
