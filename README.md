# fastfind

[![Crates.io](https://img.shields.io/crates/v/fastfind)](https://crates.io/crates/fastfind)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/fastfind)](LICENSE-MIT)

A fast, drop-in GNU `find` replacement built for AI agents and large codebases.

## Install

```sh
cargo install fastfind
```

## Usage

```sh
# list all files and directories recursively
find .

# find only files
find . -type f

# find by name glob
find . -name '*.rs'

# combine filters
find . -type f -name '*.log' -maxdepth 2

# limit depth
find . -mindepth 1 -maxdepth 3
```

## Supported flags

| Flag | Description |
| --- | --- |
| `-type f\|d\|l` | Filter by file type (file, directory, symlink) |
| `-name PATTERN` | Filter by filename glob (`*`, `?`, `[...]`) |
| `-maxdepth N` | Descend at most N levels |
| `-mindepth N` | Skip entries at depth less than N |
| `-print` | Print matching entries (default action) |

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success |
| `1` | Error (bad arguments, path not found, etc.) |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
