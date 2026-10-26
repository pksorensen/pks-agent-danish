"""Danish diagnostics from lexicon data and bounded grammatical constructions."""
import re

from language import DanishLanguage
from prose import Prose, DA_MARKERS

RULES = {
    "Danish.Spelling": "Ukendt stavemåde med et nært dansk rettelsesforslag.",
    "Danish.UnknownWord": "Ordet findes ikke i de anvendte ordressourcer; kan være et fagord eller navn.",
    "Danish.FiniteVerb": "Et personligt grundled efterfulgt direkte af en infinitiv kan mangle nutidsbøjning.",
    "Danish.AfterModal": "Efter et modalverbum bruges normalt infinitiv uden at.",
    "Danish.PredicateAgreement": "Et tillægsord som omsagn skal passe til grundleddets tal og køn.",
    "Danish.ArticleGender": "Artikel og navneord skal passe sammen i køn.",
    "Danish.AdjectiveAgreement": "Et tillægsord foran et navneord skal have den passende bøjning.",
    "Danish.RepeatedWord": "Et funktionsord står to gange i træk; kontrollér om gentagelsen er tilsigtet.",
}
SUBJECTS = {"jeg", "du", "han", "hun", "vi", "de"}
MODALS = {"kan", "kunne", "skal", "skulle", "vil", "ville", "må", "måtte", "bør", "burde"}
ADVERBS = {"ikke", "aldrig", "altid", "ofte", "gerne", "bare", "også", "stadig", "nu", "jo", "nok", "virkelig", "helt", "meget"}
DEFINITE = {"den", "det", "de", "denne", "dette", "disse", "samme", "min", "mit", "mine", "din", "dit", "dine", "sin", "sit", "sine", "vores", "deres", "hans", "hendes"}
QUANTIFIERS = {"meget", "lidt", "mere", "mindre", "flere", "færre", "nok", "samme", "egen", "eget", "egne", "andet", "andre", "mange", "eneste", "hele"}
REPEATED = {"og", "til", "på", "med", "af", "fra", "ved", "ikke", "eller", "har", "kan", "skal", "vil"}


def distance(left, right):
    """Bounded strings only; used for candidate ranking, never as a spellchecker."""
    row = list(range(len(right) + 1))
    for i, a in enumerate(left, 1):
        new = [i]
        for j, b in enumerate(right, 1):
            new.append(min(new[-1] + 1, row[j] + 1, row[j-1] + (a != b)))
        row = new
    return row[-1]


def match_case(original, replacement):
    if original.isupper():
        return replacement.upper()
    if original[:1].isupper():
        return replacement[:1].upper() + replacement[1:]
    return replacement


def internal_edit(original, corrected):
    """A narrow typo hypothesis: insert/delete inside a word, preserving both ends."""
    shorter, longer = sorted((original, corrected), key=len)
    delta = len(longer) - len(shorter)
    if delta not in {1, 2}:
        return False
    return any(longer[:i] + longer[i+delta:] == shorter for i in range(2, len(longer)-delta-1))


class Checker:
    def __init__(self, language=None, allow_words=()):
        try:
            import spacy
            self.nlp = spacy.load("da_core_news_sm", exclude=["ner"])
        except (ImportError, OSError) as exc:
            raise ValueError("Dansk analysemodel mangler. Kør scripts/setup.py og brug bin/danish.") from exc
        self.language = language or DanishLanguage()
        self.allow_words = {w.casefold() for w in allow_words}

    def check(self, source, markdown=True, uncertain=False, checks="all"):
        doc = Prose(source, markdown)
        results = []
        language = self.language
        parsed = self.nlp(doc.masked) if checks != "spelling" else []
        tokens = {token.idx: token for token in parsed}

        def token(word, *parts):
            found = tokens.get(word.start) if word else None
            return found if found is not None and found.text == word.text and (not parts or found.pos_ in parts) else None

        def emit(rule, word, message, replacements=(), severity="warning"):
            if checks == "spelling" and rule not in {"Danish.Spelling", "Danish.UnknownWord"}:
                return
            if doc.ignored(rule, word.start) or word.lower in self.allow_words:
                return
            line, span = doc.location(word.start, word.end)
            values = list(dict.fromkeys(match_case(word.text, r) for r in replacements))
            results.append({"Check": rule, "Severity": severity, "Message": message,
                            "Line": line, "Span": span, "Match": word.text,
                            "Suggestions": values, "Source": "COR 1.5.1.0 / Stavekontrolden 2.9.101",
                            "Start": word.start, "End": word.end})

        def noun_features(word):
            analyses = [(lemma, tag) for lemma, tag in language.analyses(word, "sb.")
                        if ".gen" not in tag and ".sms" not in tag]
            if not analyses:
                return None
            features = {(tag.split(".")[1], "pl" if ".pl." in tag else "sg") for _, tag in analyses}
            return next(iter(features)) if len(features) == 1 else None

        def adjective_target(word, gender, number, definite=False):
            if word in QUANTIFIERS:
                return []
            analyses = language.analyses(word, "adj.")
            if not analyses:
                return []
            if any("kompar" in tag for _, tag in analyses):
                return []
            superlative = any("superl" in tag for _, tag in analyses)
            if superlative and not definite:
                return []  # Predicative superlatives can be uninflected: de er bedst.
            if superlative:
                expected = {"adj.superl.pl", "adj.superl.sg.best"}
            elif definite:
                expected = {"adj.sg.best", "adj.pl"}
            elif number == "pl":
                expected = {"adj.pl"}
            else:
                expected = {"adj.sg.ubest." + gender}
            if any(tag in expected for _, tag in analyses):
                return []
            return language.inflections(word, lambda tag: tag in expected)

        for words in doc.segments():
            # Marker-free fragments still support grammar, but unknown words need Danish context.
            danish_context = sum(w.lower in DA_MARKERS for w in words) >= 1
            for index, word in enumerate(words):
                text = word.lower
                previous = words[index-1] if index else None
                following = words[index+1] if index+1 < len(words) else None
                if (previous and previous.lower == text and text in REPEATED
                        and doc.adjacent(previous, word)):
                    first_token, second_token = token(previous), token(word)
                    different_roles = first_token is not None and second_token is not None and first_token.pos_ == "VERB" and second_token.pos_ == "ADP"
                    if not different_roles:
                        emit("Danish.RepeatedWord", word, f"'{word.text}' står to gange i træk; kontrollér gentagelsen.")

                # Verb morphology: bound the subject/verb relation to adjacent words.
                subject_token = token(word, "PRON")
                if text in SUBJECTS and subject_token is not None and subject_token.dep_ == "nsubj":
                    cursor = index + 1
                    while cursor < len(words) and words[cursor].lower in ADVERBS and doc.adjacent(words[cursor-1], words[cursor]):
                        cursor += 1
                    if cursor < len(words) and doc.adjacent(words[cursor-1], words[cursor]):
                        verb = words[cursor]
                        if verb.lower in MODALS and token(verb, "AUX") is not None:
                            cursor += 1
                            while cursor < len(words) and words[cursor].lower in ADVERBS and doc.adjacent(words[cursor-1], words[cursor]):
                                cursor += 1
                            if cursor < len(words) and doc.adjacent(words[cursor-1], words[cursor]):
                                target = words[cursor]
                                tags = [tag for _, tag in language.analyses(target.lower, "vb.")]
                                if token(target, "VERB", "AUX") is not None and any(tag.startswith("vb.præs.") for tag in tags) and not any(tag.startswith("vb.inf.") for tag in tags):
                                    forms = language.inflections(target.lower, lambda t: t == "vb.inf.akt")
                                    if forms:
                                        emit("Danish.AfterModal", target, f"Efter '{verb.text}' forventes infinitiv; overvej '{forms[0]}'.", forms)
                        else:
                            tags = [tag for _, tag in language.analyses(verb.lower, "vb.")]
                            # 'de' can also introduce an adjective/noun phrase.
                            ambiguous_de = text == "de" and (language.analyses(verb.lower, "sb.") or language.analyses(verb.lower, "adj."))
                            verb_token = token(verb)
                            if verb_token is not None and verb_token.pos_ not in {"VERB", "AUX"} and language.analyses(verb.lower, "sb."):
                                verb_token = None
                            has_auxiliary = (verb_token is not None and any(child.dep_ in {"aux", "aux:pass"} for child in verb_token.children)) or (previous and previous.lower in MODALS)
                            linked_subject = verb_token is not None and subject_token.head == verb_token
                            if linked_subject and not has_auxiliary and not ambiguous_de and "vb.inf.akt" in tags and not any("præs" in t or "præt" in t for t in tags):
                                forms = language.inflections(verb.lower, lambda t: t == "vb.præs.akt")
                                if forms:
                                    emit("Danish.FiniteVerb", verb, f"Efter grundleddet '{word.text}' forventes et bøjet udsagnsord; overvej '{forms[0]}'.", forms)

                if following and text in {"en", "et", "den", "det"} and doc.adjacent(word, following):
                    article_token = token(word, "DET")
                    noun_token = token(following, "NOUN")
                    linked = article_token is not None and noun_token is not None and article_token.head == noun_token
                    next_word = words[index+2] if index+2 < len(words) else None
                    # An English noun modifier is not the head: 'et computer vision-projekt'.
                    next_token = tokens.get(next_word.start) if next_word else None
                    noun_modifier = next_word and doc.adjacent(following, next_word) and next_token is not None and next_token.pos_ in {"NOUN", "PROPN"}
                    foreign_modifier = next_word and doc.adjacent(following, next_word) and language.english_word(next_word.text) and not language.analyses(next_word.lower)
                    incomplete_word = source[following.end:following.end+1] in {"/", "-"}
                    ambiguous_noun = any("inf" in tag or "præs" in tag or "præt" in tag for _, tag in language.analyses(following.lower, "vb."))
                    dummy_pronoun = text in {"den", "det"} and previous and previous.lower in {"er", "var", "bliver", "blev"}
                    linked = linked and not noun_modifier and not foreign_modifier and not incomplete_word and not ambiguous_noun and not dummy_pronoun
                    features = noun_features(following.lower) if linked else None
                    if features and features[1] == "sg":
                        gender = "fk" if text in {"en", "den"} else "itk"
                        if gender != features[0]:
                            replacement = {"en": "et", "et": "en", "den": "det", "det": "den"}[text]
                            emit("Danish.ArticleGender", word, f"'{following.text}' har {features[0]}-køn i COR; overvej '{replacement}'.", [replacement])

                if following and doc.adjacent(word, following):
                    adjective_token = token(word, "ADJ")
                    noun_token = token(following, "NOUN")
                    linked = adjective_token is not None and noun_token is not None and adjective_token.head == noun_token and adjective_token.dep_ == "amod"
                    features = noun_features(following.lower) if linked and source[following.end:following.end+1] not in {"/", "-"} else None
                    # Words such as 'arbejde' can be nouns or verbs. Abstain on that ambiguity.
                    if features and not any("inf" in tag or "præs" in tag or "præt" in tag for _, tag in language.analyses(following.lower, "vb.")) and not language.analyses(following.lower, "adj."):
                        # The parser sometimes changes morphology to accommodate a typo.
                        # Prefer the actual surface form's COR analyses over predicted morphology.
                        definite_noun = all(".best" in tag for _, tag in language.analyses(following.lower, "sb.") if ".gen" not in tag)
                        explicit = False
                        cursor = index - 1
                        # Follow a short modifier chain, without crossing punctuation or a clause.
                        while cursor >= max(0, index-4) and re.fullmatch(r'[ \t*"“”«»]*', source[words[cursor].end:words[cursor+1].start]):
                            prefix = words[cursor]
                            if prefix.lower in {"slags", "type", "form", "art"}:
                                break
                            possessive = (prefix.lower.endswith(("'s", "’s")) or
                                          any(".gen" in tag for _, tag in language.analyses(prefix.lower, "sb.")) or
                                          (prefix.lower.endswith("s") and token(prefix, "NOUN", "PROPN") is not None))
                            if prefix.lower in DEFINITE or possessive:
                                explicit = True
                                break
                            if prefix.lower in {"mest", "mindst", "her", "der", "første", "anden"} or language.analyses(prefix.lower, "adj.") or token(prefix, "ADV") is not None:
                                cursor -= 1
                            else:
                                break
                        inverted_subject = previous and previous.lower in {"den", "det", "de"} and index >= 2 and words[index-2].lower in {"er", "var", "bliver", "blev"}
                        prior_adjective = previous and token(previous, "ADJ") is not None
                        definite = definite_noun or explicit
                        if inverted_subject and adjective_target(text, *features, True):
                            continue  # Could be an inverted subject: "er den ren dødvægt?"
                        # Longer adjective chains and possessives require a larger phrase analysis.
                        if not definite and (prior_adjective or any(child.dep_ == "nmod:poss" for child in noun_token.children)):
                            continue
                        forms = adjective_target(text, *features, definite)
                        if forms:
                            emit("Danish.AdjectiveAgreement", word, f"Bøjningen skal passe til '{following.text}'; overvej '{forms[0]}'.", forms)

                if following and following.lower in {"er", "var", "bliver", "blev"} and doc.adjacent(word, following):
                    features = ("fk", "pl") if text in {"vi", "de"} else noun_features(text)
                    if text in {"jeg", "du", "han", "hun"}:
                        features = ("fk", "sg")
                    cursor = index + 2
                    while cursor < len(words) and words[cursor].lower in ADVERBS and doc.adjacent(words[cursor-1], words[cursor]):
                        cursor += 1
                    if features and cursor < len(words) and doc.adjacent(words[cursor-1], words[cursor]):
                        adjective = words[cursor]
                        subj_token = token(word, "NOUN", "PRON")
                        adj_token = token(adjective, "ADJ")
                        after = words[cursor+1] if cursor+1 < len(words) else None
                        # A following noun makes this a noun phrase: 'modellerne er god reklame'.
                        noun_phrase = bool(after and doc.adjacent(adjective, after) and language.analyses(after.lower, "sb."))
                        linked = subj_token is not None and adj_token is not None and subj_token.dep_ == "nsubj" and subj_token.head == adj_token
                        idiom = adjective.lower == "vant" or (adjective.lower == "klar" and after and after.lower == "over")
                        participle = language.analyses(adjective.lower, "vb.")
                        prepositional = (subj_token is not None and any(child.dep_ == "case" for child in subj_token.children)) or any(w.lower in {"at", "for", "af", "med", "til", "om"} for w in words[max(0,index-3):index])
                        definite_subject = text in SUBJECTS or features[1] == "pl" or any(".best" in t for _, t in language.analyses(text, "sb."))
                        if linked and definite_subject and not noun_phrase and not idiom and not participle and not prepositional:
                            forms = adjective_target(adjective.lower, *features)
                            if forms:
                                emit("Danish.PredicateAgreement", adjective, f"Tillægsordet skal passe til '{word.text}'; overvej '{forms[0]}'.", forms)

                # Unknown words: keep uncertain candidates out of the normal error stream.
                if (checks == "grammar" or not danish_context or text in self.allow_words or len(text) < 4 or len(text) > 40
                        or not word.text.isalpha() or word.text.isupper()
                        or any(c.isupper() for c in word.text[1:]) or language.spelled(word.text)
                        or language.english_word(word.text)):
                    continue
                if word.text[0].isupper():
                    continue  # Names and brands require an explicit decision, not a guessed correction.
                # Productive negation and English loan inflections aren't proven spelling errors.
                productive = any(text.startswith(prefix) and len(text) > len(prefix)+3 and language.spelled(text[len(prefix):])
                                 for prefix in ("u", "un"))
                for suffix in ("ende", "erne", "ede", "ene", "ens", "er", "et", "en", "es", "e", "s"):
                    if text.endswith(suffix) and len(text) > len(suffix)+3:
                        stem = text[:-len(suffix)]
                        stems = {stem, stem[:-1] if len(stem)>1 and stem[-1] == stem[-2] else stem}
                        productive |= any(language.english_word(s) for s in stems)
                choices = [s for s in language.known_misspellings.get(text, ()) if s[:2] == text[:2]] if not productive else []
                candidates = [s.lower() for s in language.suggestions(word.text) if s.isalpha() and s.lower() != text]
                near = sorted({s for s in candidates if distance(text, s) <= (1 if len(text) < 7 else 2)},
                              key=lambda s: (distance(text, s), s))
                choices = sorted(set(choices)) or [s for s in near if not productive and len(text) >= 7 and s in language.forms
                                                      and text[:4] == s[:4] and text[-3:] == s[-3:] and internal_edit(text, s)
                                                      and distance(text, s) == min(distance(text, n) for n in near)]
                if len(choices) > 1:
                    choices = []  # Competing lexical corrections require context we cannot prove.
                if choices:
                    best_distance = min(distance(text, s) for s in choices)
                    best = [s for s in choices if distance(text, s) == best_distance]
                    if best:
                        message = (f"Ordet genkendes ikke; overvej '{best[0]}'." if len(best) == 1
                                   else "Mulig stavefejl; vælg rettelsen ud fra sammenhængen.")
                        emit("Danish.Spelling", word, message, best + [s for s in choices if s not in best][:3])
                        continue
                if uncertain:
                    emit("Danish.UnknownWord", word, "Ukendt ord; kan være et navn, fagord eller en stavefejl.", candidates[:5], "suggestion")

        # One spelling/grammar rule may propose the same span. Keep distinct diagnoses explicit.
        return sorted(results, key=lambda a: (a["Start"], a["Check"]))
