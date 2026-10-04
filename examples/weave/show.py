"""Prints a results file as a table: one row per run, the keys given as filters.

    uv run python show.py results.jsonl task=digits depth=8
"""

import json
import sys

COLUMNS = ["method", "optimizer", "rate", "depth", "hidden", "act", "kinds", "init_scale", "batch",
           "epochs", "seed"]


def main():
    filters = dict(a.split("=") for a in sys.argv[2:])
    for line in open(sys.argv[1]):
        r = json.loads(line)
        if any(str(r.get(k)) != v for k, v in filters.items()):
            continue
        head = " ".join(f"{r[k]}" for k in COLUMNS)
        seconds = r.get("seconds", 0)
        mem = r.get("rss", 0) / 2**20
        print(f"{head} | train {r['train_metrics']['acc']:.3f}/{r['train_metrics']['ce']:.3f} "
              f"test {r['test_metrics']['acc']:.3f}/{r['test_metrics']['ce']:.3f} "
              f"curve {r['curve'][0]:.3f}..{r['curve'][-1]:.4f} | {seconds:.1f}s {mem:.0f}MB")


if __name__ == "__main__":
    main()
