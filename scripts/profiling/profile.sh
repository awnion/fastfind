#!/bin/sh
set -eu
output=${1:-/results}
binary=${2:-/work/bin/baseline}
mkdir -p "$output"
{
    uname -a
    cat /etc/os-release
    rustc --version
    perf --version
    /usr/bin/find --version | head -1
    git -C /opt/FlameGraph rev-parse HEAD
    printf 'RUSTFLAGS=%s\n' "$RUSTFLAGS"
} > "$output/environment.txt"
for mode in ${PROFILE_MODES:-list miss metadata depth printf}; do
    perf stat -x , -e task-clock,context-switches,cpu-migrations,page-faults \
        -o "$output/$mode.stat.csv" -- \
        python3 /src/scripts/profiling/workload.py run --binary "$binary" \
        --trees large --modes "$mode" --repeat 30 > "$output/$mode.stat-timing.json"
    perf record -q -e cpu-clock:u -F 997 --call-graph fp \
        -o "$output/$mode.data" -- \
        python3 /src/scripts/profiling/workload.py run --binary "$binary" \
        --trees large --modes "$mode" --repeat 30 \
        > "$output/$mode.profile-timing.json" 2> "$output/$mode.record.log"
    perf report -i "$output/$mode.data" --stdio --no-children \
        --call-graph none --sort comm,dso,symbol --percent-limit 0.5 \
        > "$output/$mode.report.txt" 2> "$output/$mode.report.log"
    perf script -i "$output/$mode.data" 2> "$output/$mode.script.log" \
        | /opt/FlameGraph/stackcollapse-perf.pl > "$output/$mode.folded"
    /opt/FlameGraph/flamegraph.pl --hash --title "fastfind: $mode (user CPU)" \
        --countname nanoseconds "$output/$mode.folded" > "$output/$mode.svg"
    printf 'Finished %s\n' "$mode"
done
