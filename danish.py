#!/usr/bin/env python3
"""Local Danish grammar/spelling checker with optional declarative Vale styles."""
import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

from scripts.common import ROOT, VERSION, vale_path
from prose import Prose

PROFILES = {
    "dansk": "dansk.ini",
    "klart-sprog": "klart-sprog.ini",
    "agentics": "agentics.ini",
    "experimental": "experimental.ini",
}


def rule_records():
    return [json.loads(p.read_text(encoding="utf-8")) for p in sorted((ROOT / "rules").glob("*.json"))]


def run_vale(paths, config, stdin=None, extension=".md"):
    binary = vale_path()
    if not binary.is_file():
        raise ValueError("Vale mangler. Kør: python3 scripts/install_vale.py")
    version = subprocess.run([str(binary), "--version"], capture_output=True, text=True, check=True)
    if version.stdout.strip() != f"vale version {VERSION}":
        raise ValueError(f"Forventede Vale {VERSION}, fik: {version.stdout.strip()}")
    command = [str(binary), "--config", str(config), "--output=JSON", "--no-exit", "--minAlertLevel=suggestion"]
    if stdin is not None:
        command += [f"--ext={extension}"]
    result = subprocess.run(command + paths, input=stdin, capture_output=True, text=True)
    if result.returncode:
        raise ValueError(result.stderr.strip() or result.stdout.strip() or "Vale kunne ikke læse teksten.")
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise ValueError("Vale returnerede ugyldig JSON.") from exc


def check_document(checker, source, markdown=True, uncertain=False, checks="all", profile="dansk", name="stdin.md"):
    alerts = checker.check(source, markdown, uncertain, checks)
    # Vale remains the declarative style backend. Send only the extracted prose.
    prose = Prose(source, markdown)
    masked = ["\n" if c == "\n" else " " for c in source]
    for words in prose.segments():
        start, end = words[0].start, words[-1].end
        masked[start:end] = prose.masked[start:end]
    # Markdown has already been masked. Plain-text input avoids a second parser
    # interpreting the preserved indentation as code or losing source positions.
    styles = run_vale([], ROOT / "config" / PROFILES[profile], "".join(masked), extension=".txt") if checks != "grammar" else {}
    for items in styles.values():
        for alert in items:
            if checks == "spelling" and not alert["Check"].startswith("Dansk."):
                continue
            line, (start, end) = alert["Line"], alert["Span"]
            lines = source.splitlines()
            if not 1 <= line <= len(lines) or lines[line-1][start-1:end] != alert["Match"]:
                raise ValueError(f"Ugyldig tekstplacering fra Vale i {name}:{line}; rapporten afbrydes.")
            offset = prose.starts[line-1] + start - 1
            if alert["Check"].startswith("Dansk."):
                alerts = [a for a in alerts if not (a["Line"] == line and a["Span"] == alert["Span"] and a["Check"] == "Danish.Spelling")]
            if prose.ignored(alert["Check"], offset) or alert["Match"].lower() in checker.allow_words:
                continue
            alert["Start"] = offset
            alert["End"] = offset + len(alert["Match"])
            alert["Suggestions"] = alert.get("Action", {}).get("Params") or []
            if alert["Check"].startswith("Dansk."):
                replacement = re.match(r"Stavefejl: skriv '([^']+)'", alert["Message"])
                if replacement:
                    alert["Suggestions"] = [replacement[1]]
            alerts.append(alert)
    return sorted(alerts, key=lambda a: (a["Line"], a["Span"][0], a["Check"]))


def lint(args):
    if "-" in args.paths and args.paths != ["-"]:
        raise ValueError("Standard input (-) skal bruges alene.")
    paths = [] if args.paths == ["-"] else args.paths
    for path in paths:
        target = Path(path)
        if not target.is_file() or target.suffix.lower() not in {".txt", ".md"}:
            raise ValueError(f"Forventede en eksisterende .txt- eller .md-fil: {path}")
    stdin = sys.stdin.read() if args.paths == ["-"] else None
    if args.engine == "vale":
        diagnostics = run_vale(paths, ROOT / "config" / PROFILES[args.profile], stdin)
    else:
        from checker import Checker
        allowed = []
        if args.dictionary:
            allowed = [line.strip() for line in Path(args.dictionary).read_text(encoding="utf-8").splitlines()
                       if line.strip() and not line.lstrip().startswith("#")]
        checker = Checker(allow_words=allowed)
        diagnostics = {}
        try:
            documents = [(path, Path(path).read_text(encoding="utf-8"), Path(path).suffix.lower() == ".md") for path in paths]
            if stdin is not None:
                documents = [("stdin.md", stdin, True)]
            for name, source, markdown in documents:
                diagnostics[name] = check_document(checker, source, markdown, args.include_uncertain, args.checks, args.profile, name)
        finally:
            checker.language.close()
    if args.format == "json":
        print(json.dumps(diagnostics, ensure_ascii=False, indent=2))
    else:
        for filename, alerts in diagnostics.items():
            for alert in alerts:
                print(f"{filename}:{alert['Line']}:{alert['Span'][0]}: "
                      f"{alert['Severity']}[{alert['Check']}]: {alert['Message']}")
                if alert.get("Suggestions"):
                    print("  forslag: " + ", ".join(alert["Suggestions"]))
        if not any(diagnostics.values()):
            print("Ingen fund med den valgte regelpakke.")
    levels = {"suggestion": 0, "warning": 1, "error": 2}
    return int(any(levels[a["Severity"]] >= levels[args.fail_on]
                   for alerts in diagnostics.values() for a in alerts))


def new_rule(identifier):
    if not re.fullmatch(r"Experimental\.[A-Z][A-Za-z0-9]+", identifier):
        raise ValueError("Nye regler starter som Experimental.Regelnavn (ASCII, stort begyndelsesbogstav).")
    name = identifier.split(".")[1]
    yaml_path = ROOT / "styles" / "Experimental" / f"{name}.yml"
    record_path = ROOT / "rules" / f"{identifier}.json"
    if yaml_path.exists() or record_path.exists():
        raise ValueError(f"Reglen findes allerede: {identifier}")
    record = {
        "id": identifier, "category": "style", "status": "experimental",
        "summary": "Udfyld reglens formål", "explanation": "Udfyld forklaring og afgrænsning.",
        "source": "TODO", "source_note": "Angiv en kilde eller en begrundet stilbeslutning.",
        "exceptions": ["Tilføj kendte undtagelser."], "autofix": False,
        "cases": [
            {"text": "Her er EKSEMPELMOENSTER.", "matches": ["EKSEMPELMOENSTER"]},
            {"text": "Her er en korrekt formulering.", "matches": []},
            {"text": "Skriv `EKSEMPELMOENSTER` i koden.", "matches": []},
        ],
    }
    yaml_path.parent.mkdir(parents=True, exist_ok=True)
    yaml_path.write_text('extends: existence\nmessage: "Udfyld en hjælpsom forklaring til \'%s\'."\n'
                         'level: warning\nignorecase: true\ntokens:\n  - EKSEMPELMOENSTER\n', encoding="utf-8")
    record_path.write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Oprettet {identifier}. Udfyld kilde, regel og eksempler; kør derefter testene.")


def main():
    parser = argparse.ArgumentParser(description="pks-agent-danish — lokale, forklarlige regler for dansk tekst")
    sub = parser.add_subparsers(dest="command", required=True)
    check = sub.add_parser("lint", help="Kontrollér Markdown eller ren tekst")
    check.add_argument("paths", nargs="+", help=".md/.txt-filer eller - for standard input")
    check.add_argument("--profile", choices=PROFILES, default="dansk")
    check.add_argument("--format", choices=["text", "json"], default="text")
    check.add_argument("--fail-on", choices=["suggestion", "warning", "error"], default="error")
    check.add_argument("--engine", choices=["hybrid", "vale"], default="hybrid", help="hybrid = dansk ordbog, COR-grammatik og Vale-stil")
    check.add_argument("--include-uncertain", action="store_true", help="Vis også ukendte ord uden et nært rettelsesforslag")
    check.add_argument("--dictionary", help="Lokal ordliste, ét tilladt ord pr. linje")
    check.add_argument("--checks", choices=["all", "spelling", "grammar"], default="all")
    sub.add_parser("rules", help="Vis alle regler, også tilvalg")
    explain = sub.add_parser("explain", help="Forklar en regel")
    explain.add_argument("id")
    rule = sub.add_parser("rule", help="Opret en eksperimentel regel")
    action = rule.add_subparsers(dest="action", required=True)
    new = action.add_parser("new")
    new.add_argument("id")
    args = parser.parse_args()
    try:
        if args.command == "lint":
            return lint(args)
        if args.command == "rule":
            new_rule(args.id)
            return 0
        records = rule_records()
        if args.command == "rules":
            from checker import RULES
            for identifier, explanation in RULES.items():
                print(f"{identifier} (experimental): {explanation}")
            for record in records:
                print(f"{record['id']} ({record['status']}): {record['summary']}")
            return 0
        record = next((r for r in records if r["id"] == args.id), None)
        if record is None:
            from checker import RULES
            if args.id in RULES:
                print(f"{args.id}: {RULES[args.id]}\n\nKilde: COR 1.5.1.0 og Stavekontrolden.\n"
                      "Reglen arbejder med afgrænsede ordsekvenser og er eksperimentel.\n"
                      "Se docs/grammar.md for afgrænsning, undtagelser og bidrag.\n"
                      "Ingen automatisk rettelse; vurder altid forslaget i sammenhængen.")
                return 0
            raise ValueError(f"Ukendt regel: {args.id}")
        print(f"{record['id']}: {record['summary']}\n\n{record['explanation']}\n\n"
              f"Kilde: {record['source']}\n{record['source_note']}\n\n"
              f"Undtagelser: {' '.join(record['exceptions'])}\n"
              "Rettelsesforslag skal vurderes i kontekst; CLI'en ændrer ingen filer.")
        return 0
    except (ValueError, OSError, subprocess.SubprocessError) as exc:
        print(f"Fejl: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
