#!/usr/bin/env python3
"""Install pinned language data, retaining the upstream license notice."""
import hashlib
import json
import sys
import urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from language import resource_dir
from scripts.common import ROOT


def main():
    target = resource_dir()
    target.mkdir(parents=True, exist_ok=True)
    records = json.loads((ROOT / "resources.lock.json").read_text())
    for record in records:
        path = target / record["file"]
        if path.exists() and hashlib.sha256(path.read_bytes()).hexdigest() == record["sha256"]:
            print(f"OK {record['file']}")
            continue
        with urllib.request.urlopen(record["source"], timeout=60) as response:
            data = response.read(record["bytes"] + 1)
        if hashlib.sha256(data).hexdigest() != record["sha256"]:
            raise SystemExit(f"SHA-256 matcher ikke: {record['file']}")
        temporary = path.with_suffix(path.suffix + ".download")
        temporary.write_bytes(data)
        temporary.replace(path)
        print(f"Installeret {record['file']}")


if __name__ == "__main__":
    main()
