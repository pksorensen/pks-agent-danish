#!/usr/bin/env python3
"""Regression and CLI contracts. Quality evidence remains explicitly developmental."""
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from checker import Checker
from danish import check_document
from scripts.common import DATA, ROOT
from scripts.evaluate import quality


class CheckerTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.checker = Checker()

    @classmethod
    def tearDownClass(cls):
        cls.checker.language.close()

    def test_quality_regressions(self):
        for row in quality(self.checker)["rows"]:
            if row["text"] == "Hun gav et ærlig svar.":
                continue  # Measured separately as an expected failure, never omitted from evaluation.
            with self.subTest(text=row["text"]):
                self.assertTrue(row["passed"], row)

    @unittest.expectedFailure
    def test_known_missed_ambiguous_noun(self):
        # "svar" is both singular/plural, an adjective, and an imperative in COR.
        findings = self.checker.check("Hun gav et ærlig svar.")
        self.assertTrue(any(a["Check"] == "Danish.AdjectiveAgreement" for a in findings))

    def test_real_world_false_positive_regressions(self):
        controls = [
            "Det er den rigtige konklusion.", "Det er det egentlige produkt.",
            "Den bedst mulige løsning virker.", "Den billigst mulige bro virker.",
            "Den \"rigtige linje\" er her.", "Er det god kørsel?",
            "Ellers er den ren dødvægt.", "Vi bruger en typet kontrakt.",
            "Traefik kører på serveren.", "Vi læser stdout fra processen.",
            "Hetzner holder serveren kørende.", "Den triggerer en hændelse.",
        ]
        for text in controls:
            with self.subTest(text=text):
                self.assertEqual(self.checker.check(text), [])

    def test_source_offsets_and_suppression(self):
        text = ('---\ntitle: "Han plejedede at komme."\nslug: plejedede\n---\n'
                '\n```sessionstory\nHan plejedede at komme.\n```\n'
                '\nÅse siger: Han plejedede at komme.\n'
                '\n<!-- vale Danish.Spelling = NO -->\nHan plejedede at komme.\n'
                '<!-- vale Danish.Spelling = YES -->\n'
                'Vi læser `plejedede` på https://example.com/plejedede\n'
                'Han læser [plejedede](https://example.com/plejedede).\n')
        alerts = self.checker.check(text)
        self.assertEqual([a["Line"] for a in alerts], [2, 10, 16])
        for a in alerts:
            self.assertEqual(text[a["Start"]:a["End"]], a["Match"])
            start, end = a["Span"]
            self.assertEqual(text.splitlines()[a["Line"]-1][start-1:end], a["Match"])

    def test_unknown_words_are_separate(self):
        self.assertEqual(self.checker.check("Vi bruger xyzqwerty til det."), [])
        alerts = self.checker.check("Vi bruger xyzqwerty til det.", uncertain=True)
        self.assertEqual([(a["Check"], a["Severity"]) for a in alerts], [("Danish.UnknownWord", "suggestion")])

    def test_legacy_rule_suppression_and_masked_metadata(self):
        text = ('---\ntitle: "Hun har interresse."\n---\n'
                '<!-- vale Dansk.Interesse = NO -->\nHun har interresse.\n'
                '<!-- vale Dansk.Interesse = YES -->\n')
        alerts = check_document(self.checker, text)
        self.assertEqual([(a["Check"], a["Line"]) for a in alerts], [("Dansk.Interesse", 2)])
        # Backticks are literal characters in a .txt file, not code delimiters.
        plain = check_document(self.checker, "Hun har `ABONNOMENT`.", markdown=False)
        self.assertEqual([a["Check"] for a in plain], ["Dansk.Abonnement"])

    def cli(self, *args, text=None):
        return subprocess.run([sys.executable, str(ROOT / "danish.py"), *args],
                              input=text, capture_output=True, text=True)

    def test_hybrid_cli(self):
        result = self.cli("lint", "-", "--format", "json", "--fail-on", "warning",
                          text="Hun har interresse. Modellerne er dygtig.")
        self.assertEqual(result.returncode, 1, result.stderr)
        alerts = json.loads(result.stdout)["stdin.md"]
        self.assertEqual({a["Check"] for a in alerts}, {"Dansk.Interesse", "Danish.PredicateAgreement"})
        result = self.cli("lint", "-", "--checks", "grammar", "--format", "json",
                          text="Hun har interresse. Modellerne er dygtig.")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([a["Check"] for a in json.loads(result.stdout)["stdin.md"]], ["Danish.PredicateAgreement"])

    def test_dictionary_and_no_input_writes(self):
        DATA.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=DATA) as directory:
            path = Path(directory) / "dansk tekst.md"
            original = "Han plejedede at komme."
            path.write_text(original)
            dictionary = Path(directory) / "ord.txt"
            dictionary.write_text("# Intentionally quoted spelling\nplejedede\n")
            result = self.cli("lint", str(path), "--dictionary", str(dictionary), "--format", "json")
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(json.loads(result.stdout)[str(path)], [])
            self.assertEqual(path.read_text(), original)


if __name__ == "__main__":
    unittest.main(verbosity=2)
