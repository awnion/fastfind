# fastfind

[![Crates.io](https://img.shields.io/crates/v/fastfind)](https://crates.io/crates/fastfind)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/fastfind)](LICENSE-MIT)

A fast, drop-in GNU `find` replacement built for AI agents and large codebases.
**91% GNU find compatibility** with 1.6-1.8x better performance.

## Why

GNU `find` is single-threaded. `fd` is fast but incompatible with `find` syntax.
`fastfind` is a drop-in `find` replacement: same flags, same output, parallel traversal.

AI coding agents (Claude Code, Cursor, aider) shell out to `find` constantly.
Symlink `fastfind` as `find` and everything speeds up with zero config changes.

## Install

```sh
cargo install fastfind
```

The binary is named `find`. To use as a drop-in replacement:

```sh
ln -sf $(which find) ~/.local/bin/find   # adjust PATH priority as needed
```

## Usage

```sh
# all files and directories recursively
find .

# files only, by name
find . -type f -name '*.rs'

# complex expressions with operators
find . \( -name '*.log' -o -name '*.tmp' \) -mtime +7 -delete

# exec, like GNU find
find . -type f -name '*.txt' -exec grep -l TODO {} +

# depth-limited search
find . -maxdepth 2 -type d
```

## Features

**Tests** -- `-name`, `-iname`, `-path`, `-ipath`, `-wholename`, `-iwholename`, `-lname`, `-ilname`, `-regex`, `-iregex`, `-regextype`, `-type` (f/d/l/b/c/p/s with comma-separated multi-type), `-xtype`, `-empty`, `-size`, `-perm`, `-readable`, `-writable`, `-executable`, `-user`, `-group`, `-uid`, `-gid`, `-nouser`, `-nogroup`, `-mtime`, `-mmin`, `-atime`, `-amin`, `-ctime`, `-cmin`, `-newer`, `-anewer`, `-cnewer`, `-newerXY`, `-used`, `-fstype`, `-inum`, `-samefile`, `-links`, `-true`, `-false`, `-daystart`

**Actions** -- `-print`, `-print0`, `-printf`, `-ls`, `-fls`, `-fprint`, `-fprint0`, `-fprintf`, `-exec` (`;` and `+`), `-execdir` (`;` and `+`), `-ok`, `-okdir`, `-delete`, `-prune`, `-quit`

**Options** -- `-H`/`-L`/`-P`, `-depth`/`-d`, `-maxdepth`, `-mindepth`, `-xdev`/`-mount`, `-noleaf`, `-ignore_readdir_race`/`-noignore_readdir_race`, `-warn`/`-nowarn`

**Operators** -- `( expr )`, `! expr`/`-not`, `-a`/`-and`, `-o`/`-or`, `,` (comma/list)

See [GNU_FIND_COVERAGE.md](GNU_FIND_COVERAGE.md) for the full compatibility matrix and [GNU_FIND_COMPAT.md](GNU_FIND_COMPAT.md) for the remaining ~9%.

## Performance

- Parallel directory traversal via jwalk (rayon-based work-stealing)
- Raw byte output on Unix (skip Display/UTF-8 overhead)
- 64KB stdout buffer
- `opt-level = 3`, LTO, single codegen unit
- 1.6-1.8x faster than GNU find, 1.05-1.1x faster than fd

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success |
| `1` | Error (bad arguments, path not found, etc.) |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
