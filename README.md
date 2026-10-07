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

### From source

Requires Rust 1.99 or newer.

```sh
cargo install fastfind
```

The binary is named `find`. To use as a drop-in replacement:

```sh
mkdir -p "$HOME/.local/bin"
ln -sf "${CARGO_HOME:-$HOME/.cargo}/bin/find" "$HOME/.local/bin/find"
export PATH="$HOME/.local/bin:$PATH"
```

### From binary releases

Linux amd64 (static musl):

```sh
curl -fsSL --retry 3 https://github.com/awnion/fastfind/releases/latest/download/find-x86_64-unknown-linux-musl.tar.gz | tar xz -C /usr/local/bin
```

For Linux arm64, use `find-aarch64-unknown-linux-musl.tar.gz` instead.

macOS Apple Silicon:

```sh
curl -fsSL --retry 3 https://github.com/awnion/fastfind/releases/latest/download/find-aarch64-apple-darwin.tar.gz | tar xz -C /usr/local/bin
```

Each archive contains `find`. See [GitHub releases](https://github.com/awnion/fastfind/releases)
for all archives, including Linux GNU builds.

Future macOS releases provide prebuilt archives only for Apple Silicon.
Intel Mac users can still compile from source with `cargo install fastfind`
or the source build instructions below. Intel macOS is no longer tested in CI.

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

See the [Linux profiling report](docs/profiling/linux-arm64/README.md) for measured
hotspots, flamegraphs, optimization experiments, and reproduction steps.

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success |
| `1` | Error (bad arguments, path not found, etc.) |

## Build

Requires Rust 1.99 or newer. The repository selects the stable toolchain.

```sh
cargo build --locked --release
```

The binary is at `target/release/find`.

## Platform support and CI

We support the two latest Ubuntu LTS releases and the latest stable Debian
release on amd64 and arm64. Prebuilt macOS archives support Apple Silicon only.

All Linux archives are built on Ubuntu 24.04, the oldest supported Ubuntu
release, so GNU builds do not acquire newer glibc requirements from Ubuntu 26.04.
musl archives are statically linked. Both variants are checked using the same
packaged binaries on every supported Linux distribution and architecture.

The separate smoke matrix runs after builds in both ordinary CI and release CI:

| Runtime system | Architecture | Binary variants |
| --- | --- | --- |
| Ubuntu 24.04 | amd64, arm64 | GNU, musl |
| Ubuntu 26.04 | amd64, arm64 | GNU, musl |
| Debian 13 | amd64, arm64 | GNU, musl |
| macOS 26 | Apple Silicon (arm64) | Darwin |

These are 13 smoke combinations. Each checks `--help`, `--version`, name/type
filtering, depth limits, and recursive traversal. Ordinary CI matches the binary
version against `Cargo.toml`; release CI matches it against the release tag.
Publishing requires every smoke job to pass. This checks compatibility in the
listed environments, rather than guaranteeing every possible system setup.

CI also runs unit, integration, and doc tests on stable and nightly Rust on
Ubuntu 24.04 and 26.04 (both architectures) and macOS 26 (Apple Silicon).
Integration tests compare output with GNU find: `/usr/bin/find` on Linux and
Homebrew `gfind` on macOS. Formatting, Clippy, docs, and ordinary CI builds use
nightly; releases build and publish with stable. GitHub's macOS ARM capacity
notices can mean longer queue times, even when the jobs succeed.

## Test

Install cargo-nextest first. On macOS, install GNU find with
`brew install findutils` as well.

```sh
cargo install cargo-nextest --locked
cargo +stable nextest run --locked --release
cargo +stable test --locked --release --doc

# Also check the upcoming compiler.
cargo +nightly nextest run --locked --release
cargo +nightly test --locked --release --doc
```

Formatting requires nightly: `cargo +nightly fmt --all`.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
