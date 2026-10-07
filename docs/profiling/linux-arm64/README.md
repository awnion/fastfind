# Linux performance analysis

Created 2026-10-07.

Two small production changes are now applied: caching successful user/group name
lookups made wide-tree ownership queries 2.49x/2.64x faster, and respecting
`-maxdepth` before reading children made shallow depth-first search 4.24x faster.
They add no unsafe code. A separate cleanup removes an existing raw-pointer
workaround in formatted output. File-output buffering remains a follow-up;
the faster lazy-metadata prototype is deferred because of concurrent-tree semantics.
Ordinary parallel traversal is already fast; changing its global thread count
has mixed results.

This analysis uses commit `1832f1fed1f12e708ac782d0856dc7e0621f9901` with
**jwalk 0.9.0**, already selected by Cargo.toml and Cargo.lock. Initial profiles
and independent prototypes use this baseline. Subsequent production edits are
described separately below.

## Environment and method

- ARM64 Debian 12 container on OrbStack, 12 logical CPUs, kernel 7.0.14.
- Rust 1.99.0, GNU find 4.9.0, perf 6.1.187. The existing fastgrep profiling image
  was reused; strace and GNU time were installed before the retained benchmark runs.
- Release `opt-level=3`, LTO, one codegen unit; debug information and frame pointers
  enabled, stripping disabled. The same settings apply to every Rust variant.
- Corpus and builds lived inside the Linux filesystem. Only results and read-only
  source used macOS bind mounts. No filesystem cache eviction was performed.
- Deterministic corpus based on `benches/corpus.rs`: small = 1,300 files + 502
  directories; large = 26,000 files + 10,002 directories; wide = 50,000 empty files
  in one directory. Nested-tree files contain seven bytes. No symlinks or hidden
  files in the timing corpus; separate differential checks cover special entries.
- Each command is warmed, output is compared with GNU find, then variant order is
  randomized with a fixed seed. Wall times include process startup and discard
  output to `/dev/null`. File-output timings also target `/dev/null` and validate
  only exit status, not output contents. Each subprocess has a timeout.
- No builds, tests or profilers ran concurrently with the retained timing batches.
  The initial exploratory baseline overlapped tool installation and is excluded.

[metrics.json](metrics.json) contains every retained sample and output count.
[Environment](environment.txt), [binary hashes](build-provenance.txt), and
[reproduction instructions](../../../scripts/profiling/README.md) are retained.

## Applied changes and final validation

The final working tree contains these changes:

- [walker.rs](../../../src/walker.rs): compute whether descent is allowed before
  either sequential directory-read branch. Boundary entries are still evaluated.
- [eval.rs](../../../src/eval.rs): cache successful named UID/GID lookups separately
  in each evaluation context, which lives for one starting root. Resolution stays
  lazy, numeric predicates are unchanged, metadata is still read first, and failed
  name lookups remain retryable. Successful name-service mappings stay fixed for
  that context. Caches are empty and allocation-free until first successful lookup.
- Pass only `starting_point` and the output writer to the formatted-output helper,
  removing its existing unsafe raw-pointer alias. This is a safe borrowing cleanup,
  not a claimed speed improvement.

Final comparison, median milliseconds of 20 runs per cell, randomized order:

| Tree and command suffix | Baseline | Current | Speedup |
| --- | ---: | ---: | ---: |
| Wide, `-type f -user root` | 117.98 | 47.32 | 2.49x |
| Wide, `-type f -group root` | 117.76 | 44.60 | 2.64x |
| Large, `-type f -user root` | 85.63 | 44.32 | 1.93x |
| Large, `-type f -group root` | 83.11 | 42.94 | 1.94x |
| Large, `-depth -maxdepth 2 -type f` | 7.39 | 1.74 | 4.24x |
| Large, default listing | 20.20 | 20.22 | 1.00x |
| Large, `-type f -size +0c` | 42.53 | 42.79 | 0.99x |
| Wide, `-type f -printf '%p\n'` | 11.17 | 11.17 | 1.00x |

The remaining numeric-ID and unrelated controls varied by a few percent; no
general traversal speedup is claimed. The final [owner flamegraph](current-owner.svg)
and [syscall trace](current-owner.strace) show the cached path. The `production`
array in metrics.json contains all controls and raw samples.

Final Linux release verification: **84/84 nextest tests** and **12/12 exact-output
differential cases** against the original binary. New tests cover counted name
resolution, independent names, retry after failure, GNU ownership equivalence in
both walkers, maxdepth boundary order, and mixed `-printf`/`-fprintf` with two roots.
The output refactor's test passed before and after removing unsafe. Stable rustfmt
and `git diff --check` passed. Three scoped reviews covered traversal, evaluator/
output, and parser/matching/CLI; no speculative matching rewrite was selected.

## Confirmed walker improvements

Median milliseconds, 20 repetitions, alternating randomized variant order,
12 Rayon threads where applicable. Sequential traversal does not use Rayon.
Each patch is applied independently to the baseline.

| Tree and command suffix | Baseline | Lazy metadata | Maxdepth guard | GNU find |
| --- | ---: | ---: | ---: | ---: |
| Large, default listing | 20.49 | 20.13 | 20.31 | 52.89 |
| Large, `-depth -type f` | 48.44 | 31.23 | 47.81 | 45.56 |
| Wide, `-depth -type f` | 36.95 | 6.56 | 36.81 | 13.74 |
| Large, `-depth -maxdepth 2 -type f` | 6.82 | 6.13 | 1.59 | 4.43 |
| Wide, `-type f -size +0c` | 42.61 | 42.66 | 42.02 | 40.83 |

### 1. Reuse directory-entry types in sequential traversal

Baseline `walk_sequential_recursive` in [walker.rs](../../../src/walker.rs) calls
`symlink_metadata` for every entry, even when the expression needs only type and
path. `-depth` and expressions containing `-prune` choose this walker. The parallel
walker already obtains entry types from jwalk.

On the wide corpus, [baseline strace](wide-depth.strace) records **50,002 statx**
calls; the [prototype](wide-lazy.strace) records **2**. Wall time falls by 82.2%,
or **5.63x**. On the large nested tree the gain is **1.55x**; directory reads remain.
The [baseline depth flamegraph](depth.svg) attributes 12.38% of sampled user CPU
to libc `statx` and another 4.23% to Rust `symlink_metadata` self samples. These
percentages exclude time executing in the kernel and are not a bound on savings.
The [prototype flamegraph](lazy.svg) shifts toward directory reads.

[lazy-metadata.patch](../../../scripts/profiling/lazy-metadata.patch) passes known
regular-file/directory/symlink types from `DirEntry::file_type` into recursion.
It keeps metadata fallback for the root, special file types, failed type lookup
and `-xdev`; metadata predicates continue to load metadata on demand. On filesystems
without useful directory-entry types, Rust may still need a metadata lookup.

The patch passed **77 release nextest tests** and **12 exact-output differential
checks** covering symlinks, broken links, a FIFO, non-UTF-8 names, pruning, depth
limits, metadata tests and `-xdev`. This is not full compatibility validation:
concurrent tree mutation, filesystem boundaries and sequential `-L`/`-H` semantics
deserve explicit review before production adoption. Deferring metadata changes
when disappearing entries can be detected. Existing sequential symlink handling
is preserved, not repaired, by this experiment.

### 2. Stop directory reads at maxdepth

The sequential walker rejects a recursive call whose depth is too large, but
still opens and enumerates its parent at the maximum permitted depth.

[maxdepth.patch](../../../scripts/profiling/maxdepth.patch) guards both sequential
directory-read branches with `depth < max`. On the large tree with `-depth
-maxdepth 2`, [baseline](maxdepth-baseline.strace) versus
[prototype](maxdepth.strace) syscall counts are:

| System call | Baseline | Guard |
| --- | ---: | ---: |
| `getdents64` | 4,005 | 5 |
| `openat` | 2,006 | 6 |
| `statx` | 2,003 | 2,003 |

The isolated gain is **4.29x**, without changing the metadata strategy. This was
selected for production implementation. It also passed **77 nextest tests** and the
same **12 differential cases**. Unbounded traversal is not expected to benefit.

## Other actionable overhead

Medians below use 12 repetitions on the wide corpus, baseline with 12 threads.
These compare existing command forms, not implemented optimization patches.

### 3. Resolve named users and groups once

Baseline `Expr::User` in [evaluate](../../../src/eval.rs) calls `lookup_uid_by_name`
for each entry. `Expr::Group` has the analogous structure. Resolve a constant
name to a numeric ID once per invocation; preserve unknown-user diagnostics and
review name-service behavior. The numeric forms already avoid this work.

| Equivalent selection on this root-owned corpus | Fastfind | GNU find |
| --- | ---: | ---: |
| `-type f -user root` | 114.85 ms | 41.44 ms |
| `-type f -uid 0` | 42.21 ms | 41.04 ms |

The **2.72x** difference initially measured the opportunity; the implemented cache
is measured separately below. [Named-user strace](wide-owner.strace) has 50,009 `openat`
and 100,008 `newfstatat` calls; [numeric UID](wide-uid.strace) has 8 and 5,
respectively. Both still require about 50,000 `statx` calls for file ownership.
The [owner flamegraph](owner.svg) supports inspecting this path. Reverse lookups
for `-nouser`/`-nogroup` were not benchmarked and need their own measurements.

### 4. Keep file-output handles open and buffered

`FPrint`, `FPrint0`, `FPrintf` and `FLs` call
[open_append](../../../src/eval.rs) while evaluating each matching entry.
For `-fprint /dev/null`, [strace](wide-fprint.strace) records **50,008 openat**,
**50,008 close**, and **100,000 write** calls. Ordinary buffered `-type f` output
needs only **26 writes** on the same tree ([trace](wide-list.strace)).

`-fprint /dev/null` took **43.22 ms**; GNU find took **13.87 ms**. Ordinary fastfind
listing of files was around **9 ms** in the separate thread sweep. Retaining a
buffered writer per action/destination is promising, but that wall-time delta is
not an implemented gain. A production change needs tests for multiple actions
targeting the same file, truncation, output errors, ordering and final flush.

## Parallel traversal and thread counts

[Listing flamegraph](list.svg), [no-match flamegraph](miss.svg),
[metadata flamegraph](metadata.svg), [printf flamegraph](printf.svg).
For default listing, ordered-queue heap pops account for 11.43% combined self
samples, Rayon `par_bridge` producer work for 7.47%, contended mutex locking for
4.51%, and three ARM atomic-add helpers for 9.35%. Name matching is not the leading
hotspot in these workloads. The suffix optimization for `*.rs` already exists.

Medians of 15 runs per cell, randomized command order, baseline binary:

| Tree and workload | 1 thread | 2 threads | 4 threads | 12 threads |
| --- | ---: | ---: | ---: | ---: |
| Small, listing | 2.24 | 1.73 | 1.88 | 3.67 |
| Large, listing | 35.05 | 22.57 | 21.55 | 19.68 |
| Large, metadata | 44.71 | 35.70 | 40.94 | 42.50 |
| Large, maxdepth 2 | 1.43 | 1.53 | 1.76 | 3.97 |
| Wide, listing | 8.93 | 8.81 | 8.80 | 9.11 |

`RAYON_NUM_THREADS` already provides a way to explore this without changing code.
One Rayon worker is not the custom sequential walker. A small-tree/shallow-search
policy may help, but a global cap would regress the large listing workload here.
Metadata evaluation happens in the consuming thread, so adding traversal workers
does not parallelize the per-file metadata predicates. Moving evaluation to
workers would require separating pure predicates from ordered, stateful actions;
that is a larger change than the optimizations above.

## Limits and retained artifacts

These are synthetic, warm-cache measurements in an ARM VM, not claims about cold
storage, NFS, x86, musl or all real repositories. Very short commands show sizable
VM timing variation; use raw distributions and repeat on deployment hardware
before choosing automatic thread thresholds. Separate timing batches should not
be interpreted as a controlled before/after comparison with each other.

Perf sampled `cpu-clock:u` at 997 Hz with frame-pointer call graphs, 30 command
invocations per profile. The retained recordings report zero lost samples. Hardware
`cycles` were unsupported, so no IPC, cache-miss or branch-miss claims are made.
Flamegraph widths represent sampled user CPU, including the Python repetition
harness, not wall time or kernel execution. Low sample counts and inlining limit
fine attribution. `perf stat` CSVs retain task-clock, context switches, migrations
and faults; these cover the harness too. Strace runs are separate from timings.

Reports, SVGs, syscall summaries and raw timing samples are ready for review here.
Raw perf data, folded stacks, test/build logs and exploratory results remain in
the ignored `target/profiling/` directory. The stopped container
`fastfind-profiler-1832f1f` retains the matching binaries and corpus. The report was collected before the v0.1.4 release.
