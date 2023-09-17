# Dexter, a cbz toolbox

The repository host multiple cli (and gui) tools that help manipulating [cbz files](https://fileinfo.com/extension/cbz).
It also offers a `cbz` lib crate that exposes some of the code used internally.

Tools:

- `dexter` - cli - A simple MangaDex client
- `sinister` - gui - _in progress_ - A gui client for Dexter

- `cbz-reader` - gui - A dead simple cbz file reader
- `cbz-merge` - cli - Merge cbz files together
- `cbz-pack` - cli - pack images into a cbz file

## Dexter

This small cli allows you to search for mangas hosted on MangaDex and read them.
The target is to become a full featured wrapper for the MangaDex website, and for the moment it already provides some valuable features.

This repository also contains a small `cbz-reader` which is not meant to be used as is for now (even though you can, it might not work with all CBZ files). It's used by `dexter` after you download a manga's chapter using the `--open` flag, see the `Usage` section for more.

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

## Cbz Merge

This will look for all the Cbz archives file foundable in `path` and which file name contains `something` and merge into `output/merged_archive.cbz`:

```bash
cbz-merge --archives-glob "path/**/*something*" --outdir "output" --name "merged_archive"
```

## Cbz Pack

Takes all the `png` files under `source` and pack them into the `archive.cbz` file:

```bash
cbz-pack "source/*.png" --name archive --autosplit
```

You can also autoextract images from a pdf:

```bash
cbz-pack source.pdf --pdf --name archive
```

Options inclue:

- `--autosplit`: split in 2 landscape images
- `--contrast`: change contrast
- `--brightness`: change brightness

## Cbz Converter

Converts from \* to cbz (only pdf, mobi, and DRM-free azw3 supported for the moment):

```bash
cbz-converter "archive.azw3" --from azw3 --outdir out
```
