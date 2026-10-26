"""Local Danish lexicon and spelling engine. No network calls during analysis."""
import ctypes
import ctypes.util
import functools
import os
import sys
from collections import defaultdict
from pathlib import Path

from scripts.common import DATA


def resource_dir():
    return Path(os.environ.get("PKS_DANISH_RESOURCES", DATA / "resources"))


class DanishLanguage:
    def __init__(self):
        directory = resource_dir()
        for name in ("da_DK.aff", "da_DK.dic", "en_US.aff", "en_US.dic", "cor1.5.1.0.tsv", "corsoegehjaelp1.0.tsv"):
            if not (directory / name).is_file():
                raise ValueError("Sprogdata mangler. Kør python3 scripts/install_language.py")
        library = os.environ.get("PKS_DANISH_HUNSPELL") or ctypes.util.find_library("hunspell-1.7") or ctypes.util.find_library("hunspell")
        if not library:
            raise ValueError("Hunspell-biblioteket mangler. Linux: libhunspell-1.7-0; macOS: brew install hunspell.")
        self.lib = ctypes.CDLL(library)
        self.lib.Hunspell_create.argtypes = [ctypes.c_char_p, ctypes.c_char_p]
        self.lib.Hunspell_create.restype = ctypes.c_void_p
        self.lib.Hunspell_destroy.argtypes = [ctypes.c_void_p]
        self.lib.Hunspell_get_dic_encoding.argtypes = [ctypes.c_void_p]
        self.lib.Hunspell_get_dic_encoding.restype = ctypes.c_char_p
        self.lib.Hunspell_spell.argtypes = [ctypes.c_void_p, ctypes.c_char_p]
        self.lib.Hunspell_spell.restype = ctypes.c_int
        self.list_type = ctypes.POINTER(ctypes.c_char_p)
        self.lib.Hunspell_suggest.argtypes = [ctypes.c_void_p, ctypes.POINTER(self.list_type), ctypes.c_char_p]
        self.lib.Hunspell_suggest.restype = ctypes.c_int
        self.lib.Hunspell_free_list.argtypes = [ctypes.c_void_p, ctypes.POINTER(self.list_type), ctypes.c_int]
        self.handle = self.lib.Hunspell_create(os.fsencode(directory / "da_DK.aff"), os.fsencode(directory / "da_DK.dic"))
        if not self.handle:
            raise ValueError("Hunspell kunne ikke indlæse ordbogen.")
        self.encoding = self.lib.Hunspell_get_dic_encoding(self.handle).decode()
        self.english = self.lib.Hunspell_create(os.fsencode(directory / "en_US.aff"), os.fsencode(directory / "en_US.dic"))
        if not self.english:
            self.close()
            raise ValueError("Den engelske hjælpeordbog kunne ikke indlæses.")
        self.english_encoding = self.lib.Hunspell_get_dic_encoding(self.english).decode()
        self.forms = defaultdict(list)
        self.lemmas = defaultdict(list)
        with (directory / "cor1.5.1.0.tsv").open(encoding="utf-8") as stream:
            for line in stream:
                fields = line.rstrip("\n").split("\t")
                if len(fields) != 6:
                    continue
                identifier, lemma, gloss, tag, form, norm = fields
                if norm not in {"N", "K"}:
                    continue
                lemma_id = sys.intern(".".join(identifier.split(".")[:2]))
                tag = sys.intern(tag)
                form = form.lower()
                self.forms[form].append((lemma_id, tag))
                self.lemmas[lemma_id].append((form, tag))
        self.known_misspellings = defaultdict(set)
        with (directory / "corsoegehjaelp1.0.tsv").open(encoding="utf-8") as stream:
            for line in stream:
                fields = line.rstrip("\n").split("\t")
                if len(fields) == 4:
                    _, correct, _, alternative = fields
                    # Search-help also contains legal variants. Never treat all rows as errors.
                    if alternative.isalpha() and all(part.isalpha() and part.lower() in self.forms for part in correct.split()):
                        self.known_misspellings[alternative.lower()].add(correct.lower())

    def close(self):
        if getattr(self, "english", None):
            self.lib.Hunspell_destroy(self.english)
            self.english = None
        if self.handle:
            self.lib.Hunspell_destroy(self.handle)
            self.handle = None

    def analyses(self, word, prefix=""):
        return [(lemma, tag) for lemma, tag in self.forms.get(word.lower(), ()) if tag.startswith(prefix)]

    def inflections(self, word, target):
        answers = set()
        for lemma, _ in self.analyses(word):
            answers.update(form for form, tag in self.lemmas[lemma] if target(tag))
        return sorted(answers)

    @functools.lru_cache(maxsize=50000)
    def spelled(self, word):
        return word.lower() in self.forms or bool(self.lib.Hunspell_spell(self.handle, word.encode(self.encoding)))

    @functools.lru_cache(maxsize=10000)
    def suggestions(self, word):
        values = self.list_type()
        count = self.lib.Hunspell_suggest(self.handle, ctypes.byref(values), word.encode(self.encoding))
        try:
            return tuple(values[i].decode(self.encoding) for i in range(min(count, 8)))
        finally:
            if count:
                self.lib.Hunspell_free_list(self.handle, ctypes.byref(values), count)

    @functools.lru_cache(maxsize=50000)
    def english_word(self, word):
        try:
            encoded = word.encode(self.english_encoding)
        except UnicodeEncodeError:
            return False
        return bool(self.lib.Hunspell_spell(self.english, encoded))
