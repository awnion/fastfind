#!/usr/bin/env bash
set -euo pipefail

tar xzf "find-$TARGET.tar.gz"
if [[ "$GITHUB_REF_TYPE" == "tag" ]]; then
  expected_version="${GITHUB_REF_NAME#v}"
else
  expected_version=$(grep -m1 '^version = ' Cargo.toml | cut -d '"' -f2)
fi
test -n "$expected_version"

./find --help
version=$(./find --version)
printf '%s\n' "$version"
printf '%s\n' "$version" | grep -F "find (fastfind) $expected_version ("

mkdir -p fixture/sub
touch fixture/match.txt fixture/sub/match.txt fixture/skip.log
result=$(./find fixture -maxdepth 1 -type f -name '*.txt')
test "$result" = 'fixture/match.txt'
result=$(./find fixture -type f -name '*.txt' | sort)
expected=$(printf 'fixture/match.txt\nfixture/sub/match.txt\n')
test "$result" = "$expected"
