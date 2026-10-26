#!/usr/bin/env python3
"""Generate the human-readable rule catalog from the tested rule records."""
import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from danish import rule_records
from scripts.common import ROOT


def catalog():
    lines = ["# Regeloversigt", "", "Genereret fra `rules/*.json` med `python3 scripts/catalog.py`.",
             "Redigér regelbeskrivelserne, og generér denne fil igen.", "",
             "Startreglerne er afgrænsede og har endnu ikke eksternt sprogfagligt review.", ""]
    for rule in rule_records():
        link = rule["source"]
        if not link.startswith("https://"):
            link = "../" + link
        lines += [f"## {rule['id']}", "", rule["summary"], "",
                  f"Kategori: `{rule['category']}`. Status: `{rule['status']}`. Automatisk rettelse: nej.", "",
                  rule["explanation"], "", f"[Kilde]({link}). {rule['source_note']}", "",
                  "Undtagelser: " + " ".join(rule["exceptions"]), "",
                  f"Se [testeksemplerne](../rules/{rule['id']}.json).", ""]
    return "\n".join(lines)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    target = ROOT / "docs/rules.md"
    expected = catalog()
    if args.check:
        if not target.exists() or target.read_text(encoding="utf-8") != expected:
            raise SystemExit("Regeloversigten er forældet. Kør python3 scripts/catalog.py")
        print("Regeloversigten er ajour.")
    else:
        target.write_text(expected, encoding="utf-8")
        print(target)


if __name__ == "__main__":
    main()
