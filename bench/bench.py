#!/usr/bin/env python3
"""Run bench.lox and report wall-clock timing."""

import argparse
import subprocess
import sys
import time
import os
from pathlib import Path

# Force current working directory to repository root (parent of bench/)
# so `cargo run` and other commands execute with the repo as CWD.
ROOT = Path(__file__).resolve().parent.parent
os.chdir(ROOT)


def run_bench(release: bool, iterations: int) -> None:
    cmd = ["cargo", "run", "-p", "native"]
    if release:
        cmd.append("--release")
    cmd.extend(["--", os.path.join("bench", "bench.lox")])

    print("Running benchmark with:")
    print("  " + " ".join(cmd))
    print(f"  iterations={iterations} (first run is warmup and ignored)\n")

    total = 0.0
    last_output = None

    print("warmup run (ignored)")
    warmup_proc = subprocess.run(cmd, capture_output=True, text=True)
    if warmup_proc.returncode != 0:
        print(warmup_proc.stdout)
        print(warmup_proc.stderr, file=sys.stderr)
        raise SystemExit(f"Warmup run failed with exit code {warmup_proc.returncode}")

    for i in range(1, iterations + 1):
        start = time.perf_counter()
        proc = subprocess.run(cmd, capture_output=True, text=True)
        duration = time.perf_counter() - start
        total += duration
        last_output = proc.stdout.strip()

        if proc.returncode != 0:
            print(proc.stdout)
            print(proc.stderr, file=sys.stderr)
            raise SystemExit(f"Benchmark run failed with exit code {proc.returncode}")

        print(f"run {i}/{iterations}: {duration:.3f}s")

    print(f"\naverage wall-clock: {total / iterations:.3f}s")
    if last_output:
        print("\nlast program output:")
        print(last_output)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Time bench.lox using the native interpreter."
    )
    parser.add_argument("--release", action="store_true", help="Run the release build.")
    parser.add_argument(
        "--iterations", type=int, default=3, help="How many times to run the benchmark."
    )
    args = parser.parse_args()

    run_bench(args.release, args.iterations)
