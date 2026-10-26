"""Shared paths; generated data stays on workspace storage."""
import os
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKSPACE = next(
    (p for p in ROOT.parents if (p / "scripts/with-workspace-tmp.sh").is_file()), ROOT
)
DATA = WORKSPACE / "data" / "pks-agent-danish"
VERSION = "3.25.1"


def vale_path():
    return Path(os.environ.get("PKS_DANISH_VALE", DATA / "tools" / VERSION / "vale"))
