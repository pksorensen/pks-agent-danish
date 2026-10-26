#!/usr/bin/env python3
"""Install the isolated, pinned runtime and language resources."""
import os
import subprocess
import sys
import venv
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from scripts.common import DATA, ROOT


def main():
    environment = DATA / "venv"
    interpreter = environment / "bin" / "python"
    if not interpreter.is_file():
        if sys.version_info < (3, 12):
            raise SystemExit("Brug Python 3.12+ til setup. Systemets Python ændres ikke.")
        venv.EnvBuilder(with_pip=True).create(environment)
    temporary = DATA / "tmp"
    temporary.mkdir(parents=True, exist_ok=True)
    env = {**os.environ, "PIP_CACHE_DIR": str(DATA / "pip-cache")}
    for name in ("TMPDIR", "TMP", "TEMP"):
        env.setdefault(name, str(temporary))
    pip = subprocess.run([str(interpreter), "-m", "pip", "--version"], capture_output=True)
    if pip.returncode:
        subprocess.run([str(interpreter), "-m", "ensurepip"], check=True, env=env)
    subprocess.run([str(interpreter), "-m", "pip", "install", "-r", str(ROOT / "requirements.lock")], check=True, env=env)
    subprocess.run([str(interpreter), str(ROOT / "scripts/install_language.py")], check=True, env=env)
    subprocess.run([str(interpreter), str(ROOT / "scripts/install_vale.py")], check=True, env=env)
    subprocess.run([str(interpreter), "-c", "import spacy; spacy.load('da_core_news_sm'); print('Dansk analysemodel klar.')"], check=True, env=env)
    print(f"Klar: {ROOT / 'bin/danish'} lint tekst.md")


if __name__ == "__main__":
    main()
