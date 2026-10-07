# Linux performance profiling

Run from the repository root with Docker running. Source is mounted read-only;
builds and the corpus live in the Linux container filesystem, not the host mount.
The image pins Rust 1.99.0 and FlameGraph. The release optimization settings are
retained, with debug symbols, frame pointers and no stripping.

```sh
mkdir -p target/profiling
docker build -t fastfind-profiler:local -f scripts/profiling/Dockerfile scripts/profiling
docker run -d --name fastfind-profiler --init \
  --cap-add PERFMON --cap-add SYS_PTRACE --security-opt seccomp=unconfined \
  -v "$PWD:/src:ro" -v "$PWD/target/profiling:/results" \
  fastfind-profiler:local sleep infinity
docker exec fastfind-profiler sh -ec '
  mkdir -p /work/repo /work/bin
  git -c safe.directory=/src -C /src archive 1832f1fed1f12e708ac782d0856dc7e0621f9901 \
    -o /work/baseline.tar
  tar -C /work/repo -xf /work/baseline.tar
  cd /work/repo
  cargo build --locked --release --bin find
  cp /work/target/release/find /work/bin/baseline
  python3 /src/scripts/profiling/workload.py generate > /results/corpus.json
  python3 /src/scripts/profiling/workload.py compare \
    --threads 1 2 4 12 --repeat 15 > /results/threads.json
  sh /src/scripts/profiling/profile.sh
  sh /src/scripts/profiling/syscalls.sh
'
```

`generate` refuses to reuse an existing corpus. `compare` checks sorted output
against GNU find before measuring, warms each command, then randomizes variant
order with a fixed seed. Timings include process startup, exclude generation and
profiling overhead, discard stdout, and retain every sample. Each subprocess has
a timeout. `owner` assumes the container runs as root. `fprint` targets `/dev/null`;
its timing exposes output overhead but its empty stdout does not validate file
contents. Existing integration tests cover basic file-output correctness.

Run benchmarks with no concurrent builds, tests, profilers, or other benchmark
jobs. The corpus uses warm filesystem caches; no host or guest cache eviction is
performed. Software CPU-clock works in the tested OrbStack VM; hardware counters
may not. The graphs show user CPU only. The Python repetition harness also appears
in samples; profiler timings are not benchmark results.

## Isolated experiments

These are research patches against `1832f1f`. The setup above deliberately builds
that revision so later production edits do not invalidate the comparison. Each patch
is applied independently to a fresh source copy. Do not combine them when
reproducing the individual gains.

```sh
docker exec fastfind-profiler sh -ec '
  cargo install cargo-nextest --locked --version 0.9.129
  for variant in lazy maxdepth; do
    cp -a /work/repo /work/$variant
    cd /work/$variant
    case "$variant" in
      lazy) patch -p1 < /src/scripts/profiling/lazy-metadata.patch ;;
      maxdepth) patch -p1 < /src/scripts/profiling/maxdepth.patch ;;
    esac
    CARGO_TARGET_DIR=/work/target-$variant cargo build --locked --release --bin find
    cp /work/target-$variant/release/find /work/bin/$variant
    CARGO_TARGET_DIR=/work/target-$variant cargo nextest run --locked --release
    python3 /src/scripts/profiling/verify.py /work/bin/baseline /work/bin/$variant
  done
  python3 /src/scripts/profiling/workload.py compare \
    --variant baseline=/work/bin/baseline --variant lazy=/work/bin/lazy \
    --variant maxdepth=/work/bin/maxdepth --variant gnu=/usr/bin/find \
    --modes list depth depth_shallow metadata --repeat 20 > /results/prototypes.json
  PROFILE_MODES=depth sh /src/scripts/profiling/profile.sh /results/lazy /work/bin/lazy
  PROFILE_MODES=owner sh /src/scripts/profiling/profile.sh /results/owner /work/bin/baseline
'
```

The lazy-metadata patch passes directory-entry types to recursive traversal,
retaining metadata fallback for roots, special types, failures and `-xdev`.
Metadata-dependent expressions still request metadata lazily. Its behavior under
concurrent replacement/deletion of entries needs further review, as does existing
sequential symlink-following behavior. `verify.py` compares exact output and status
with the baseline on 12 static cases including symlinks, FIFO, non-UTF-8 names,
metadata tests, pruning and depth limits. It is a smoke check, not proof of full
GNU compatibility.

The maxdepth patch stops reading a directory when its children would be beyond
the requested depth. It applies to both sequential traversal branches.

## Compare the current working tree

After preparing the pinned baseline above, build the current source in a separate
directory and run the same commands against both binaries:

```sh
docker exec fastfind-profiler sh -ec '
  mkdir -p /work/current
  tar -C /src --exclude=target --exclude=.git --exclude=docs -cf - . \
    | tar -C /work/current -xf -
  cd /work/current
  CARGO_TARGET_DIR=/work/target-current cargo build --locked --release --bin find
  cp /work/target-current/release/find /work/bin/current
  CARGO_TARGET_DIR=/work/target-current cargo nextest run --locked --release
  python3 /src/scripts/profiling/verify.py /work/bin/baseline /work/bin/current
  python3 /src/scripts/profiling/workload.py compare \
    --variant baseline=/work/bin/baseline --variant current=/work/bin/current \
    --trees large wide --modes list metadata owner uid group gid depth_shallow printf \
    --repeat 20 > /results/production.json
  PROFILE_MODES=owner sh /src/scripts/profiling/profile.sh /results/current-owner /work/bin/current
  sh /src/scripts/profiling/syscalls.sh /results/current /work/bin/current
'
```

Install nextest as shown in the experiments section before running this sequence.

The crate documentation's GNU find table was measured separately with the v0.1.4
release candidate, after all other builds and checks had finished:

```sh
docker exec fastfind-profiler python3 /src/scripts/profiling/workload.py compare \
  --variant release=/work/bin/current --variant gnu=/usr/bin/find \
  --trees large wide --modes list name metadata owner depth depth_shallow \
  --repeat 20 > target/profiling/release-comparison.json
```

Artifacts and logs stay in the ignored `target/profiling/` directory. Stop the
container after collection; keep it if you want to inspect the matching Linux
binaries, corpus and raw perf data later:

```sh
docker stop fastfind-profiler
```

The 2026-10-07 session used the container `fastfind-profiler-1832f1f` and reused
the existing `fastgrep-profiler:local` Debian 12 image, installing `strace` and
`time` in the container. The Dockerfile reproduces that tool setup from scratch.
