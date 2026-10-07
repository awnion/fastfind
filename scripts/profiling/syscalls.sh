#!/bin/sh
set -eu
output=${1:-/results}
binary=${2:-/work/bin/baseline}
mkdir -p "$output"
for mode in list depth owner uid fprint; do
    case "$mode" in
        list) set -- -type f ;;
        depth) set -- -depth -type f ;;
        owner) set -- -type f -user root ;;
        uid) set -- -type f -uid 0 ;;
        fprint) set -- -type f -fprint /dev/null ;;
    esac
    strace -f -c -o "$output/wide-$mode.strace" "$binary" /work/corpus/wide "$@" \
        > /dev/null 2> "$output/wide-$mode.strace.log"
done
