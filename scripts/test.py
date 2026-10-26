#!/usr/bin/env python3
"""Run the tests using the installed project environment, in either checkout layout."""
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from scripts.common import DATA, ROOT

if __name__ == '__main__':
    interpreter = DATA / 'venv/bin/python'
    if not interpreter.is_file():
        raise SystemExit('Kør først python3.12 scripts/setup.py')
    raise SystemExit(subprocess.call([str(interpreter), '-m', 'unittest', 'discover',
                                      '-s', str(ROOT / 'tests'), '-v'], cwd=ROOT))
