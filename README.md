# splitx

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/splitx.svg
[crates-url]: https://crates.io/crates/splitx
[docs-badge]: https://img.shields.io/docsrs/splitx
[docs-url]: https://docs.rs/splitx/1.0.0/splitx
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/loyd/elfo/blob/master/LICENSE

splitx is a tiny rust library for splitting a text file into pieces with the size of each piece below a specified maximum number of bytes on disk.


## Usage
To use `splitx`, add this to your `Cargo.toml`:
```toml
[dependencies]
splitx = "1.0"
```

The library's `split` function has the following arguments:
```
file_path: the path of the file to be split,
max_file_size_bytes: the maximum size of each piece of the file in bytes after splitting,
num_header_lines: how many lines are the file's header. If no header lines, use 0. Header lines will be kept in each of the pieces.
output_dir: where to write the pieces of the file.
```
