"""Paths: the weave Python package and the roop binary the experiments use."""

import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "crates/roop-weave/python"))


def roop():
    """The roop binary: WEAVE_ROOP, else the release build, else the debug one."""
    if "WEAVE_ROOP" in os.environ:
        return os.environ["WEAVE_ROOP"]
    for profile in ("release", "debug"):
        binary = ROOT / "target" / profile / "roop"
        if binary.exists():
            os.environ.setdefault("ROOP_RT_LIB", str(ROOT / "target" / profile / "libroop_rt.a"))
            return str(binary)
    raise SystemExit("build roop first: cargo build --release -p roop -p roop-rt")


def library():
    """The directory of the weave and einsum modules: WEAVE_LIBRARY, else this repository's roop."""
    return os.environ.get("WEAVE_LIBRARY", str(ROOT / "roop"))
