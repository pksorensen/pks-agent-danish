#!/usr/bin/env python3
"""Reproducible development evaluation; never rewrites input documents."""
import argparse
import hashlib
import json
import sys
import time
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from checker import Checker
from danish import check_document
from scripts.common import ROOT


def quality(checker):
    cases = json.loads((ROOT / "tests/quality.json").read_text())["cases"]
    rows = []
    for case in cases:
        actual = check_document(checker, case["text"])
        expected = case["findings"]
        missing = [item for item in expected if not any(
            a["Check"] == item[0] and a["Match"] == item[1]
            and (item[2] is None or item[2] in a["Suggestions"]) for a in actual)]
        unexpected = [a for a in actual if not any(
            a["Check"] == rule and a["Match"] == word for rule, word, _ in expected)]
        rows.append({**case, "passed": not missing and not unexpected,
                     "missing": missing, "unexpected": unexpected, "actual": actual})
    return {"description": "Author-written development cases, not held out or independently reviewed.",
            "cases": len(rows), "passed": sum(r["passed"] for r in rows),
            "error_cases": sum(bool(r["findings"]) for r in rows),
            "controls": sum(not r["findings"] for r in rows),
            "rows": rows}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--blog-root", type=Path, help="Directory containing */da.md")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    start = time.monotonic()
    checker = Checker()
    report = {"quality": quality(checker), "files": [], "findings": []}
    try:
        paths = sorted(args.blog_root.glob("*/da.md")) if args.blog_root else []
        if args.blog_root and not paths:
            parser.error("No */da.md files found")
        for index, path in enumerate(paths):
            raw = path.read_bytes()
            source = raw.decode("utf-8")
            alerts = check_document(checker, source, uncertain=True, name=str(path))
            for a in alerts:
                assert source[a["Start"]:a["End"]] == a["Match"]
                left, right = a["Span"]
                assert source.splitlines()[a["Line"]-1][left-1:right] == a["Match"]
                a["file"] = str(path)
                a["context"] = source[max(0, a["Start"]-65):a["End"]+85]
            digest = hashlib.sha256(raw).hexdigest()
            assert hashlib.sha256(path.read_bytes()).hexdigest() == digest, f"Source changed: {path}"
            report["files"].append({"file": str(path), "sha256": digest})
            report["findings"].extend(alerts)
            if (index + 1) % 40 == 0:
                print(f"Checked {index+1}/{len(paths)} documents", file=sys.stderr)
        for item in report["files"]:
            assert hashlib.sha256(Path(item["file"]).read_bytes()).hexdigest() == item["sha256"]
    finally:
        checker.language.close()
    report["summary"] = {"files": len(report["files"]),
                         "by_rule": dict(Counter(a["Check"] for a in report["findings"])),
                         "warnings": sum(a["Severity"] == "warning" for a in report["findings"]),
                         "seconds": round(time.monotonic()-start, 2),
                         "source_unchanged": True,
                         "scope": "Hybrid CLI pipeline, default dansk profile, uncertain words recorded separately. No custom dictionary."}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"quality_passed": report["quality"]["passed"],
                      "quality_cases": report["quality"]["cases"], **report["summary"]}, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
