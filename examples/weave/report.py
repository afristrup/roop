"""Prints results files as a markdown table, one row per configuration, the mean and the
standard deviation over its seeds.

    uv run python report.py results/digits_depth8.jsonl [more files]
"""

import collections
import json
import math
import statistics
import sys

KEYS = ["task", "method", "optimizer", "rate", "depth", "act", "kinds", "batch", "epochs", "init_scale",
        "out_scale", "norm", "input_scale", "loss_scale", "dtype"]
DEFAULTS = {"init_scale": 1.0, "out_scale": 1.0, "norm": False, "input_scale": 1.0, "loss_scale": 1, "dtype": "float64"}


def spread(values, digits=3):
    if not all(math.isfinite(v) for v in values):
        return "nan"
    if max(abs(v) for v in values) > 1000:
        return f"{statistics.mean(values):.1e}"
    if len(values) == 1:
        return f"{values[0]:.{digits}f}"
    return f"{statistics.mean(values):.{digits}f} +- {statistics.stdev(values):.{digits}f}"


def group(paths):
    rows = collections.defaultdict(list)
    for path in paths:
        for line in open(path):
            run = json.loads(line)
            rows[tuple(run.get(k, DEFAULTS.get(k)) for k in KEYS)].append(run)
    return rows


def megabytes(runs, key):
    values = [r[key] / 2**20 for r in runs if key in r]
    return spread(values, 1) if values else "0"


def main(paths):
    print("| task | method | optimizer | rate | depth | n | test acc | test CE | train CE | final loss "
          "| cpu s | RSS MB | saved MB |")
    print("|" + " --- |" * 13)
    for key, runs in sorted(group(paths).items(), key=lambda kv: [str(x) for x in kv[0]]):
        task, method, optimizer, rate, depth = key[:5]
        if method == "plain":
            method = f"plain {key[-1]}"
        if method == "weave" and key[-2] != 1:
            method = f"weave x{key[-2]}"
        print(f"| {task} | {method} | {optimizer} | {rate} | {depth} | {len(runs)} "
              f"| {spread([r['test_metrics']['acc'] for r in runs])} "
              f"| {spread([r['test_metrics']['ce'] for r in runs])} "
              f"| {spread([r['train_metrics']['ce'] for r in runs])} "
              f"| {spread([r['curve'][-1] for r in runs], 4)} "
              f"| {spread([r.get('cpu', r.get('seconds', 0)) for r in runs], 1)} "
              f"| {megabytes(runs, 'rss')} | {megabytes(runs, 'saved')} |")


if __name__ == "__main__":
    main(sys.argv[1:])
