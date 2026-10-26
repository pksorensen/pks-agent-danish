"""Conservative source-preserving Markdown extraction for local diagnostics.

Mask syntax, never delete characters: offsets always address the original text.
This deliberately supports prose Markdown, not arbitrary MDX/JSX documents.
"""
import bisect
import re
from dataclasses import dataclass

WORDS = re.compile(r"[^\W\d_]+(?:[-'’][^\W\d_]+)*", re.UNICODE)
DA_MARKERS = set("jeg du han hun vi de det den der dem en et og men ikke som til af på med fra har havde er var blev bliver skal skulle vil ville kan kunne må måtte også fordi hvis når hvor hvad vores deres sin sine sit noget meget denne dette disse være have at mig dig sig for om så selv hele ved mere alle man jo få først nu efter før hvordan hvorfor min mit din dit uden over under ind ud eller igen sammen sådan både mellem imod gennem hver hvilket faktisk kun end her dem os jer i stedet flere andet andre hverken ingen endnu altid meget mere mest dog også nok helt lige netop samtidig derfor bliver blevet gør gjort gøre samme noget næsten dag godt gang ting måde mennesker år".split())
EN_MARKERS = set("the this that these those with from and but not are was were been being is will would can could should have has had which what when where why how your their its our you they we my of to for as it in on into without about after before than then there here just do does did don't doesn't isn't wasn't can't won't let let's I'll I've that's it's so also all any some only now one each both more most very while through between because if else by an a".lower().split())


@dataclass(frozen=True)
class Word:
    text: str
    start: int
    end: int

    @property
    def lower(self):
        return self.text.lower()


class Prose:
    def __init__(self, source, markdown=True):
        self.source = source
        self.starts = [0] + [m.end() for m in re.finditer("\n", source)]
        chars = list(source)

        def mask(start, end):
            for i in range(start, end):
                if chars[i] not in "\r\n":
                    chars[i] = " "

        self.suppressions = []
        active = {}
        for m in re.finditer(r"<!--\s*vale\s+([\w.]+)\s*=\s*(NO|YES)\s*-->", source):
            rule, toggle = m.groups()
            if toggle == "NO":
                active.setdefault(rule, m.end())
            elif rule in active:
                self.suppressions.append((rule, active.pop(rule), m.start()))
        self.suppressions += [(rule, start, len(source)) for rule, start in active.items()]

        if markdown:
            lines = source.splitlines(keepends=True)
            offset = 0
            fence = None
            frontmatter = bool(lines and lines[0].strip() == "---")
            for index, line in enumerate(lines):
                end = offset + len(line)
                if frontmatter:
                    # Only user-facing scalar metadata. All structural metadata is opaque.
                    match = re.match(r"^(title|subtitle|description):\s*[\"']?", line)
                    if match:
                        mask(offset, offset + match.end())
                    else:
                        mask(offset, end)
                    if index and line.strip() in {"---", "..."}:
                        frontmatter = False
                elif fence:
                    mask(offset, end)
                    if re.match(r"^ {0,3}" + re.escape(fence[0]) + "{" + str(fence[1]) + r",}\s*$", line):
                        fence = None
                else:
                    match = re.match(r"^ {0,3}(`{3,}|~{3,})(.*)$", line)
                    if match:
                        delimiter = match[1]
                        if delimiter[0] != "`" or "`" not in match[2]:
                            fence = (delimiter[0], len(delimiter))
                            mask(offset, end)
                    elif line.startswith(("    ", "\t")):
                        mask(offset, end)
                    elif re.match(r"^ {0,3}\[[^]]+\]:", line):
                        mask(offset, end)
                offset = end
            current = "".join(chars)
            for match in re.finditer(r"<!--[\s\S]*?(?:-->|\Z)|<(?:pre|code|script|style)\b[^>]*>[\s\S]*?</(?:pre|code|script|style)\s*>|<[^>\n]+>", current, re.I):
                mask(*match.span())
            current = "".join(chars)
            for match in re.finditer(r"(?<!`)(`+)(?!`)([\s\S]*?)(?<!`)\1(?!`)", current):
                mask(*match.span())
            current = "".join(chars)
            # Mask destinations but retain visible link labels and image alt text.
            for match in re.finditer(r"\]\((?:[^()\n]|\([^()\n]*\))*\)|\]\[[^]\n]*\]", current):
                mask(match.start() + 1, match.end())

        current = "".join(chars)
        for match in re.finditer(r"(?:https?://|www\.)\S+|[\w.+-]+@[\w.-]+\.\w+", current):
            mask(*match.span())
        self.masked = "".join(chars)

    def ignored(self, rule, offset):
        return any((name == rule or name == "Danish") and start <= offset < end
                   for name, start, end in self.suppressions)

    def location(self, start, end):
        line = bisect.bisect_right(self.starts, start) - 1
        return line + 1, [start - self.starts[line] + 1, end - self.starts[line]]

    def segments(self):
        # Line/sentence boundaries are conservative. No cross-line grammar inference.
        for match in re.finditer(r"[^\n.!?;]+", self.masked):
            words = [Word(w.group(), match.start() + w.start(), match.start() + w.end())
                     for w in WORDS.finditer(match.group())]
            if not words:
                continue
            da = sum(w.lower in DA_MARKERS for w in words)
            en = sum(w.lower in EN_MARKERS for w in words)
            # A heuristic, not language identification: abstain on English-dominated spans.
            if en >= 2 and en > da:
                continue
            yield words

    def adjacent(self, first, second):
        gap = self.source[first.end:second.start]
        return bool(re.fullmatch(r"[ \t]*(?:\*{1,2})?[ \t]*", gap))
