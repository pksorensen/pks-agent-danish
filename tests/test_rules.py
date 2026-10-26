#!/usr/bin/env python3
"""Exercise the real Vale engine, source positions, profiles and CLI contract."""
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from danish import PROFILES, rule_records, run_vale
from scripts.common import DATA, ROOT


class RulesTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        DATA.mkdir(parents=True, exist_ok=True)
        cls.records = rule_records()

    def test_catalog_contract(self):
        ids = [r["id"] for r in self.records]
        self.assertEqual(len(ids), len(set(ids)), "Regel-id'er skal være unikke")
        yaml_ids = {f"{p.parent.name}.{p.stem}" for p in (ROOT / "styles").glob("*/*.yml")}
        self.assertEqual(set(ids), yaml_ids, "Hver regel skal have metadata og eksempler")
        for record in self.records:
            with self.subTest(rule=record["id"]):
                self.assertTrue((ROOT / "rules" / f"{record['id']}.json").is_file())
                self.assertIn(record["category"], {"spelling", "grammar", "style", "terminology"})
                self.assertIn(record["status"], {"starter", "experimental", "stable"})
                self.assertIs(record["autofix"], False, "Automatisk rettelse er ikke understøttet endnu")
                for field in ["summary", "explanation", "source", "source_note", "exceptions"]:
                    self.assertTrue(record[field], field)
                    self.assertNotIn("TODO", str(record[field]))
                    self.assertNotIn("Udfyld", str(record[field]))
                source = record["source"]
                self.assertTrue(source.startswith("https://") or (ROOT / source.split("#")[0]).is_file())
                self.assertGreaterEqual(sum(bool(c["matches"]) for c in record["cases"]), 2)
                self.assertGreaterEqual(sum(not c["matches"] for c in record["cases"]), 3)
                if record["status"] == "experimental":
                    self.assertTrue(record["id"].startswith("Experimental."))

    def test_rule_examples(self):
        count = 0
        with tempfile.TemporaryDirectory(dir=DATA, prefix="rule-tests-") as directory:
            work = Path(directory)
            for record in self.records:
                identifier = record["id"]
                pack = identifier.split(".")[0]
                config = work / "test.ini"
                settings = [f"StylesPath = {ROOT / 'styles'}", "[*.md]", f"BasedOnStyles = {pack}"]
                settings += [f"{other['id']} = NO" for other in self.records
                             if other["id"].startswith(pack + ".") and other["id"] != identifier]
                config.write_text("\n".join(settings) + "\n", encoding="utf-8")
                paths = []
                for index, case in enumerate(record["cases"]):
                    path = work / f"case-{index}.md"
                    path.write_text(case["text"], encoding="utf-8")
                    paths.append(str(path))
                result = run_vale(paths, config)
                for path, case in zip(paths, record["cases"]):
                    with self.subTest(rule=identifier, text=case["text"]):
                        alerts = result.get(path, [])
                        self.assertEqual([a["Check"] for a in alerts], [identifier] * len(case["matches"]))
                        self.assertEqual([a["Match"] for a in alerts], case["matches"])
                        for alert in alerts:
                            line = case["text"].splitlines()[alert["Line"] - 1]
                            start, end = alert["Span"]
                            self.assertEqual(line[start - 1:end], alert["Match"], "Position i originaltekst")
                            self.assertEqual(alert["Severity"], "error" if record["category"] == "spelling" else "warning")
                    count += 1
        print(f"\n{count} regeleksempler kontrolleret med Vale.")

    def test_corpus(self):
        expected = json.loads((ROOT / "tests/corpus/expected.json").read_text(encoding="utf-8"))
        for profile, files in expected.items():
            paths = [str(ROOT / "tests/corpus" / name) for name in files]
            result = run_vale(paths, ROOT / "config" / PROFILES[profile])
            for path, checks in files.items():
                with self.subTest(profile=profile, file=path):
                    alerts = result.get(str(ROOT / "tests/corpus" / path), [])
                    self.assertEqual(sorted(a["Check"] for a in alerts), sorted(checks))
                    print(f"Korpus {profile}/{path}: {len(alerts)} fund")

    def cli(self, *args, text=None, cwd=ROOT):
        # These are the original backend's characterization tests. Hybrid has its own suite.
        if args and args[0] == "lint":
            args = (*args, "--engine", "vale")
        return subprocess.run([sys.executable, str(ROOT / "danish.py"), *args], input=text,
                              capture_output=True, text=True, cwd=cwd)

    def test_profiles_and_exit_codes(self):
        text = "Vi fremsender hermed vores features med interresse."
        for profile, checks in [
            ("dansk", {"Dansk.Interesse"}),
            ("klart-sprog", {"Dansk.Interesse", "KlartSprog.FremsenderHermed"}),
            ("agentics", {"Dansk.Interesse", "KlartSprog.FremsenderHermed", "Agentics.Features"}),
        ]:
            with self.subTest(profile=profile):
                result = self.cli("lint", "--profile", profile, "--format", "json", "-", text=text)
                self.assertEqual(result.returncode, 1, result.stderr)
                self.assertEqual({a["Check"] for alerts in json.loads(result.stdout).values() for a in alerts}, checks)
        self.assertEqual(self.cli("lint", "--profile", "agentics", "-", text="Vores features.").returncode, 0)
        self.assertEqual(self.cli("lint", "--profile", "agentics", "--fail-on", "warning", "-", text="Vores features.").returncode, 1)
        self.assertEqual(self.cli("lint", "-", text="Ingen stavefejl her.").returncode, 0)
        self.assertEqual(self.cli("lint", "does-not-exist.md").returncode, 2)
        self.assertEqual(self.cli("explain", "Unknown.Rule").returncode, 2)
        self.assertEqual(self.cli("rule", "new", "../Invalid").returncode, 2)
        self.assertEqual(self.cli("explain", "Dansk.Interesse").returncode, 0)

    def test_markdown_source_positions_and_no_writes(self):
        with tempfile.TemporaryDirectory(dir=DATA, prefix="cli-tests-") as directory:
            path = Path(directory) / "tekst med mellemrum.md"
            original = "# En overskrift\n\nÅse har interresse.\n\n`interresse`\n"
            path.write_text(original, encoding="utf-8")
            result = self.cli("lint", "--format", "json", path.name, cwd=directory)
            self.assertEqual(result.returncode, 1, result.stderr)
            alerts = [a for items in json.loads(result.stdout).values() for a in items]
            self.assertEqual(len(alerts), 1)
            self.assertEqual(alerts[0]["Line"], 3)
            self.assertEqual(alerts[0]["Span"], [9, 18])
            self.assertEqual(path.read_text(encoding="utf-8"), original)
            plain = Path(directory) / "tekst.txt"
            plain.write_text("Åse har interresse.", encoding="utf-8")
            self.assertEqual(self.cli("lint", str(plain)).returncode, 1)


if __name__ == "__main__":
    unittest.main(verbosity=2)
