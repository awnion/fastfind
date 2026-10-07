//! Fastfind - parallel filesystem search with GNU find-style expressions.
//!
//! The installed binary is named `find`. It supports name and path patterns,
//! file types, metadata predicates, boolean expressions, depth limits, pruning,
//! formatted output, and execution actions. Ordinary directory traversal uses
//! jwalk workers; depth-first and prune expressions use a sequential walker.
//!
//! # Command-line usage
//!
//! ```sh
//! find . -type f -name '*.rs'
//! find . -maxdepth 2 -type d
//! find . -name target -prune -o -type f -print
//! find . -type f -name '*.txt' -exec grep -l TODO {} +
//! ```
//!
//! Install with `cargo install fastfind`. The minimum supported Rust version is
//! 1.99. The [compatibility matrix](https://github.com/awnion/fastfind/blob/v0.1.4/GNU_FIND_COVERAGE.md)
//! and [remaining differences](https://github.com/awnion/fastfind/blob/v0.1.4/GNU_FIND_COMPAT.md)
//! describe the supported subset; not every GNU find behavior is implemented.
//!
//! # Benchmarks: fastfind versus GNU find
//!
//! Measured **2026-10-08** using the v0.1.4 release candidate and GNU find 4.9.0.
//! The environment was a Debian 12 ARM64 Docker container on OrbStack with 12
//! logical CPUs, Rust 1.99.0, and warm filesystem caches. Builds and corpus were
//! inside the Linux filesystem. Release optimization settings were retained
//! (`opt-level=3`, LTO, one codegen unit), with symbols and frame pointers enabled
//! for consistency with the profiling builds. Fastfind used 12 Rayon threads.
//!
//! The large tree contains 26,000 seven-byte files and 10,002 directories; the
//! wide tree contains 50,000 empty files in one directory. Corpus generation
//! follows the repository's directory-tree benchmarks.
//!
//! | Workload | fastfind | GNU find | GNU/fastfind |
//! | --- | ---: | ---: | ---: |
//! | Large: list all | 21.29 ms | 47.21 ms | 2.22x |
//! | Large: -name '*.rs' | 21.70 ms | 50.64 ms | 2.33x |
//! | Large: -type f -size +0c | 41.32 ms | 60.19 ms | 1.46x |
//! | Large: -type f -user root | 42.28 ms | 62.38 ms | 1.48x |
//! | Large: -depth -type f | 54.80 ms | 46.23 ms | 0.84x |
//! | Large: -depth -maxdepth 2 -type f | 1.83 ms | 4.32 ms | 2.36x |
//! | Wide: list all | 9.27 ms | 13.20 ms | 1.42x |
//! | Wide: -depth -type f | 39.51 ms | 14.92 ms | 0.38x |
//!
//! Values are median wall times from 20 runs per command, including process startup,
//! with stdout redirected to `/dev/null`. Each command was warmed before timing;
//! variant order was randomized with a fixed seed, and sorted output was checked
//! against GNU find. No builds or profilers ran concurrently with these timings.
//! `GNU/fastfind` is GNU find time divided by fastfind time; below 1 means GNU find
//! is faster. Ratios use unrounded medians. These synthetic, warm-cache VM results
//! do not establish a general speedup on other filesystems, machines, or corpora.
//!
//! The flat-tree depth-first case remains slower than GNU find because the
//! sequential walker reads metadata for every entry. Removing those reads is
//! an experiment, not part of v0.1.4. This release instead avoids reading beyond
//! `-maxdepth` and caches successful user/group name resolution per starting root.
//!
//! See the [full profiling report and flamegraphs](https://github.com/awnion/fastfind/blob/v0.1.4/docs/profiling/linux-arm64/README.md),
//! [raw comparison samples](https://github.com/awnion/fastfind/blob/v0.1.4/docs/profiling/linux-arm64/release-comparison.json),
//! and [reproduction commands](https://github.com/awnion/fastfind/blob/v0.1.4/scripts/profiling/README.md).
//!
//! # Library example
//!
//! [`parser::parse`] accepts find-style arguments without the executable name and
//! returns an [`expr::Config`]. [`walker::walk`] evaluates that configuration and
//! writes results to the supplied writer. Remember to flush buffered output.
//!
//! ```no_run
//! use std::io::BufWriter;
//! use std::io::Write;
//! use std::io::{self};
//!
//! fn main() -> io::Result<()> {
//!     let config =
//!         fastfind::parser::parse([".", "-maxdepth", "3", "-type", "f", "-name", "*.rs"])
//!             .map_err(io::Error::other)?;
//!     let stdout = io::stdout();
//!     let mut output = BufWriter::new(stdout.lock());
//!     fastfind::walker::walk(&config, &mut output)?;
//!     output.flush()
//! }
//! ```

pub mod cli;
pub mod eval;
pub mod expr;
pub mod parser;
pub mod walker;
