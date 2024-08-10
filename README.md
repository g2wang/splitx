# splitx

[![MIT licensed][mit-badge]][mit-url]

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/loyd/elfo/blob/master/LICENSE

splitx is a tiny rust library for splitting a text file into pieces which the size of each piece below a specified maximum in bytes.

**Note: It's not published to crates.io yet.**

## Usage
  Call the library's `split` function with the specified arguments.
```
  split (
    file_path: the path to the file to be split,
    max_file_size_bytes: the maximum size of each piece of the file in bytes after split,
    num_header_lines: how many lines are the file's header. If no header lines, use 0. Header lines will be kept in each of the pieces.
    output_dir: where to wite the pieces of the files.
    )
```
