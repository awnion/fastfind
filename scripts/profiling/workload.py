#!/usr/bin/env python3
"""Deterministic Linux-local corpus, differential checks and wall-clock timings."""
import argparse
import json
import os
from pathlib import Path
import random
import statistics
import subprocess
import time

BASE = Path('/work/corpus')
MODES = {
    'list': [],
    'type': ['-type', 'f'],
    'name': ['-name', '*.rs'],
    'miss': ['-name', '*absent*'],
    'metadata': ['-type', 'f', '-size', '+0c'],
    'shallow': ['-maxdepth', '2', '-type', 'f'],
    'depth': ['-depth', '-type', 'f'],
    'printf': ['-type', 'f', '-printf', '%p\n'],
    'owner': ['-type', 'f', '-user', 'root'],
    'uid': ['-type', 'f', '-uid', '0'],
    'group': ['-type', 'f', '-group', 'root'],
    'gid': ['-type', 'f', '-gid', '0'],
    'fprint': ['-type', 'f', '-fprint', '/dev/null'],
    'depth_shallow': ['-depth', '-maxdepth', '2', '-type', 'f'],
}


def generate():
    assert not BASE.exists(), 'Use a fresh corpus directory'
    counts = {}
    for kind, modules in [('small', 100), ('large', 2000)]:
        root = BASE / kind
        for i in range(modules):
            current = root / 'src' / f'module_{i:04}'
            (current / 'tests').mkdir(parents=True)
            for name in ['lib.rs', 'mod.rs', 'utils.rs', f'tests/test_{i:04}.rs']:
                (current / name).write_text('sample\n')
            for depth in range(3):
                current /= f'sub_{depth}'
                current.mkdir()
                for name in [f'file_{i:04}.rs', f'data_{i:04}.log', f'config_{i:04}.toml']:
                    (current / name).write_text('sample\n')
        counts[kind] = {'files': modules * 13, 'directories': modules * 5 + 2}
    root = BASE / 'wide'
    root.mkdir()
    for i in range(50000):
        (root / f'file_{i:06}.rs').touch()
    counts['wide'] = {'files': 50000, 'directories': 1}
    return counts


def command(binary, tree, mode):
    return [binary, str(BASE / tree), *MODES[mode]]


def run(binary, tree, mode, threads):
    env = dict(os.environ, RAYON_NUM_THREADS=str(threads))
    start = time.perf_counter_ns()
    subprocess.run(command(binary, tree, mode), env=env, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE,
                   check=True, timeout=60)
    return (time.perf_counter_ns() - start) / 1e6


def main():
    p = argparse.ArgumentParser()
    p.add_argument('action', choices=['generate', 'compare', 'run'])
    p.add_argument('--binary', default='/work/bin/baseline')
    p.add_argument('--variant', action='append', default=[], help='NAME=PATH')
    p.add_argument('--trees', nargs='+', default=['small', 'large', 'wide'])
    p.add_argument('--modes', nargs='+', choices=MODES, default=list(MODES))
    p.add_argument('--threads', nargs='+', type=int, default=[12])
    p.add_argument('--repeat', type=int, default=12)
    args = p.parse_args()
    if args.action == 'generate':
        print(json.dumps(generate(), indent=2))
        return
    if args.action == 'run':
        values = [run(args.binary, args.trees[0], args.modes[0], args.threads[0])
                  for _ in range(args.repeat)]
        print(json.dumps(values))
        return
    variants = dict(v.split('=', 1) for v in args.variant) or {
        'baseline': args.binary, 'gnu': '/usr/bin/find'}
    result = []
    rng = random.Random(1832)
    for tree in args.trees:
        for mode in args.modes:
            expected = sorted(subprocess.check_output(command('/usr/bin/find', tree, mode), timeout=60).splitlines())
            cells = [(name, binary, threads) for name, binary in variants.items()
                     for threads in (args.threads if name != 'gnu' else [1])]
            samples = {(name, threads): [] for name, _, threads in cells}
            for name, binary, threads in cells:
                actual = sorted(subprocess.check_output(command(binary, tree, mode),
                                env=dict(os.environ, RAYON_NUM_THREADS=str(threads)), timeout=60).splitlines())
                assert actual == expected, (name, tree, mode, threads, len(actual), len(expected))
                run(binary, tree, mode, threads)
            for _ in range(args.repeat):
                rng.shuffle(cells)
                for name, binary, threads in cells:
                    samples[name, threads].append(run(binary, tree, mode, threads))
            for name, _, threads in cells:
                values = samples[name, threads]
                result.append(dict(tree=tree, mode=mode, variant=name, threads=threads,
                                   median_ms=statistics.median(values), samples_ms=values,
                                   output_records=len(expected)))
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
