#!/usr/bin/env python3
"""Differential smoke checks for isolated walker prototypes (Linux only)."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile

baseline, candidate = sys.argv[1:]
cases = [
    ['-depth', '-print0'],
    ['-depth', '-type', 'f', '-print0'],
    ['-depth', '-type', 'p', '-print0'],
    ['-depth', '-type', 'l', '-print0'],
    ['-depth', '-size', '+0c', '-print0'],
    ['-depth', '-empty', '-print0'],
    ['-depth', '-xdev', '-print0'],
    ['-depth', '-maxdepth', '0', '-print0'],
    ['-depth', '-maxdepth', '1', '-print0'],
    ['-depth', '-maxdepth', '2', '-mindepth', '1', '-print0'],
    ['-name', 'skip', '-prune', '-o', '-print0'],
    ['-maxdepth', '1', '-name', 'skip', '-prune', '-o', '-print0'],
]
with tempfile.TemporaryDirectory(prefix='fastfind-verify-') as tmp:
    root = Path(tmp)
    (root / 'skip' / 'nested').mkdir(parents=True)
    (root / 'keep').mkdir()
    (root / 'keep' / 'data').write_text('data')
    (root / 'empty').touch()
    (root / 'link').symlink_to('keep', target_is_directory=True)
    (root / 'broken').symlink_to('missing')
    os.mkfifo(root / 'fifo')
    fd = os.open(os.fsencode(tmp) + b'/nonutf8-\xff', os.O_CREAT | os.O_WRONLY, 0o600)
    os.close(fd)
    for args in cases:
        outputs = []
        for binary in [baseline, candidate]:
            run = subprocess.run([binary, tmp, *args], capture_output=True, timeout=10)
            outputs.append((run.returncode, run.stdout, run.stderr))
        assert outputs[0] == outputs[1], args
print(f'{len(cases)} differential cases passed: {candidate}')
