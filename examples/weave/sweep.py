"""Runs run.py once per line of stdin (its arguments) and appends the JSON lines to a file.

    printf 'digits --method plain\ndigits --method weave\n' | uv run --extra torch python sweep.py results.jsonl

Each run is its own process, so the peak memory of one does not hide in another.
"""

import shlex
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent


def main():
    target = sys.argv[1]
    for line in sys.stdin:
        if not line.strip():
            continue
        done = subprocess.run([sys.executable, str(HERE / "run.py"), *shlex.split(line)],
                              capture_output=True, text=True)
        if done.returncode != 0:
            print(f"FAILED {line.strip()}\n{done.stderr[-600:]}", file=sys.stderr)
            continue
        with open(target, "a") as out:
            out.write(done.stdout)
        print("ok", line.strip(), flush=True)


if __name__ == "__main__":
    main()
