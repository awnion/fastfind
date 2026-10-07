# Changelog

## v0.1.4

Released 2026-10-08.

- Avoid reading children beyond `-maxdepth` in sequential depth-first and prune traversal.
- Cache successful `-user` and `-group` name lookups per starting root, while preserving lazy evaluation and retrying failed lookups.
- Remove the unsafe raw-pointer workaround from `-printf` output; use safe field borrowing instead.
- Add regression coverage for owner lookup caching, depth boundaries, and `-printf`/`-fprintf` with multiple starting roots (84 tests total).
- Add crate-level docs.rs documentation with usage, measured GNU find comparisons, and a checked library example.
- Add reproducible Linux profiling with perf, strace, flamegraphs, raw measurements, and isolated optimization experiments. On the measured warm-cache ARM64 corpus, named ownership queries improved 2.5-2.6x and shallow depth-first traversal improved 4.2x; results depend on workload.
- Update dependencies, including jwalk 0.9, and require Rust 1.99 or newer.
- Expand CI to stable/nightly Rust on Ubuntu 24.04, Ubuntu 26.04, and Apple Silicon macOS 26. Verify packaged GNU/musl Linux binaries on both Ubuntu LTS releases and Debian 13, for amd64 and arm64.
- Build Linux releases on Ubuntu 24.04 for older glibc compatibility. Prebuilt macOS releases now target Apple Silicon; Intel macOS remains source-build only.

## v0.1.3

- Replace hand-written `--help` with clap-generated output
- Structured help: sections for symlink options, global options, tests, actions, operators
- Bash-style syntax highlighting in examples (commands, flags, patterns, placeholders)
- Add `-exec`/`-execdir` usage examples with `\;` vs `+` explained
- Colors auto-stripped when piped (via clap's anstream)
- Clean up dead code in `cli.rs`, remove backward-compat shim from `lib.rs`

## v0.1.2

**Full GNU find expression engine** -- 91% feature coverage, up from ~8%.

- Recursive descent parser with expression tree (AND/OR/NOT/grouping/comma)
- Expression evaluator with short-circuit semantics
- 77 integration tests with GNU find parity (up from 25)

**Tests:**
- `-iname`, `-path`/`-wholename`, `-ipath`/`-iwholename`, `-lname`, `-ilname`
- `-regex`, `-iregex` with `-regextype` (emacs, posix-basic, posix-extended, grep, egrep)
- `-type` extended to all POSIX types (b/c/p/s), comma-separated multi-type
- `-xtype`, `-empty`, `-size` (c/w/b/k/M/G with +/- prefixes)
- `-perm` (exact, -all, /any in octal), `-readable`, `-writable`, `-executable`
- `-user`, `-group`, `-uid`, `-gid`, `-nouser`, `-nogroup`
- `-mtime`, `-mmin`, `-atime`, `-amin`, `-ctime`, `-cmin`, `-newer`, `-anewer`, `-cnewer`, `-newerXY`, `-used`
- `-fstype`, `-inum`, `-samefile`, `-links`
- `-true`, `-false`, `-daystart`

**Actions:**
- `-print0`, `-printf` (full format specifiers), `-ls`, `-fls`
- `-fprint`, `-fprint0`, `-fprintf`
- `-exec` (`;` and `+`), `-execdir` (`;` and `+`), `-ok`, `-okdir`
- `-delete` (with implicit `-depth`), `-prune`, `-quit`

**Options:**
- `-H`/`-L`/`-P` symlink following modes
- `-depth`/`-d`, `-xdev`/`-mount`, `-noleaf`
- `-ignore_readdir_race`/`-noignore_readdir_race`
- `-warn`/`-nowarn`

**Operators:**
- `( expr )`, `! expr`/`-not`, `-a`/`-and`, `-o`/`-or`, `,` (comma/list)

**Performance:**
- `opt-level = 3` (speed over size)
- Byte-level glob matching on Unix (skip UTF-8 validation)
- 1.6-1.8x faster than GNU find, 1.05-1.1x faster than fd

## v0.1.1

- Parallel directory traversal via jwalk (rayon-based work-stealing)
- Raw byte output on Unix (skip Display formatting overhead)
- 64KB stdout buffer (vs 8KB default)
- Use d_type from readdir — no extra stat() calls for type filtering
- 1.4-1.6x faster than GNU find across all benchmarks
- Add `--version` and `--help` flags
- Add criterion benchmarks with baseline comparison

## v0.1.0

- Initial release
- Minimal GNU find replacement: `-type f|d|l`, `-name`, `-maxdepth`, `-mindepth`, `-print`
- Deterministic sorted output
- 25 integration tests with GNU find parity
