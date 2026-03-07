# Changelog

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
