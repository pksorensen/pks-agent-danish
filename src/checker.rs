use crate::{
    diagnostic::{Diagnostic, match_case},
    language::Language,
    parser::{Parsed, Parser},
    prose::{DA, Prose, Word, has},
    rules::Rules,
};
use anyhow::{Result, bail};
use std::{collections::HashSet, path::Path};
pub const CORE: &[(&str, &str)] = &[
    (
        "Danish.Spelling",
        "Ukendt stavemåde med et nært dansk rettelsesforslag.",
    ),
    (
        "Danish.UnknownWord",
        "Ukendt ord; kan være et navn, fagord eller en stavefejl.",
    ),
    (
        "Danish.FiniteVerb",
        "Et personligt grundled efterfulgt af infinitiv kan mangle nutidsbøjning.",
    ),
    (
        "Danish.AfterModal",
        "Efter et modalverbum bruges normalt infinitiv uden at.",
    ),
    (
        "Danish.PredicateAgreement",
        "Et tillægsord som omsagn skal passe til grundleddets tal og køn.",
    ),
    (
        "Danish.ArticleGender",
        "Artikel og navneord skal passe sammen i køn.",
    ),
    (
        "Danish.AdjectiveAgreement",
        "Tillægsordet skal passe til navneordets tal, køn og bestemthed.",
    ),
    (
        "Danish.RepeatedWord",
        "Et funktionsord står to gange i træk; kontrollér gentagelsen.",
    ),
];
const SUBJECTS: &str = "jeg du han hun vi de";
const MODALS: &str = "kan kunne skal skulle vil ville må måtte bør burde";
const ADVERBS: &str = "ikke aldrig altid ofte gerne bare også stadig nu jo nok virkelig helt meget";
const DEFINITE: &str = "den det de denne dette disse samme min mit mine din dit dine sin sit sine vores deres hans hendes";
const QUANTIFIERS: &str = "meget lidt mere mindre mest mindst flere færre nok samme egen eget egne andet andre mange eneste hele";
const REPEATED: &str = "og til på med af fra ved ikke eller har kan skal vil";
const COPULA: &str = "er var bliver blev";
pub struct Options {
    pub markdown: bool,
    pub uncertain: bool,
    pub checks: String,
    pub profile: String,
    pub allowed: HashSet<String>,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            markdown: true,
            uncertain: false,
            checks: "all".into(),
            profile: "dansk".into(),
            allowed: HashSet::new(),
        }
    }
}
pub struct Checker {
    pub language: Language,
    pub parser: Parser,
    pub rules: Rules,
}
fn is_finite_verb(l: &Language, w: &str) -> bool {
    l.analyses(w, "vb.")
        .iter()
        .any(|a| a.tag.contains("inf") || a.tag.contains("præs") || a.tag.contains("præt"))
}
fn lexical_noun(l: &Language, word: &str) -> bool {
    !l.analyses(word, "sb.").is_empty()
        && l.analyses(word, "adj.").is_empty()
        && l.analyses(word, "vb.").is_empty()
        && l.analyses(word, "pron").is_empty()
        && l.analyses(word, "adv").is_empty()
}
fn incomplete_noun(l: &Language, source: &str, word: &Word) -> bool {
    let tail = source[word.end..].trim_start_matches(['*', '_']);
    tail.starts_with(['/', '-', '#', '+', '@'])
        || tail.chars().next().is_some_and(|c| c.is_ascii_digit())
        || (tail.starts_with('.') && l.forms.contains_key(&format!("{}.", word.lower)))
}
fn noun_modifier(l: &Language, parsed: &Parsed, word: &Word) -> bool {
    word.lower.contains('-')
        || lexical_noun(l, &word.lower)
        || parsed.token(word, &["NOUN", "PROPN"]).is_some()
        || (l.english(&word.text) && l.analyses(&word.lower, "").is_empty())
}
impl Checker {
    pub fn load(resources: &Path, model: Option<&Path>, rules: Option<&Path>) -> Result<Self> {
        Ok(Self {
            language: Language::load(resources)?,
            parser: Parser::load(model)?,
            rules: Rules::load(rules)?,
        })
    }
    fn adjective_target(
        &self,
        word: &str,
        gender: &str,
        number: &str,
        definite: bool,
    ) -> Vec<String> {
        let l = &self.language;
        let analyses = l.analyses(word, "adj.");
        if has(QUANTIFIERS, word)
            || analyses.is_empty()
            || analyses.iter().any(|a| a.tag.contains("kompar"))
        {
            return vec![];
        }
        let superlative = analyses.iter().any(|a| a.tag.contains("superl"));
        if superlative && !definite {
            return vec![];
        }
        let expected = if superlative {
            vec!["adj.superl.pl".into(), "adj.superl.sg.best".into()]
        } else if definite {
            vec!["adj.sg.best".into(), "adj.pl".into()]
        } else if number == "pl" {
            vec!["adj.pl".into()]
        } else {
            vec![format!("adj.sg.ubest.{gender}")]
        };
        if analyses.iter().any(|a| expected.contains(&a.tag)) {
            vec![]
        } else {
            l.inflections(word, |t| expected.iter().any(|e| e == t))
        }
    }
    pub fn check(&self, source: &str, o: &Options) -> Result<Vec<Diagnostic>> {
        if source.contains('\0') {
            bail!("Teksten indeholder NUL, som analysemodellen ikke understøtter");
        }
        let doc = Prose::new(source, o.markdown);
        let parsed = if o.checks == "spelling" {
            Parsed::empty()
        } else {
            self.parser.parse(&doc.masked)?
        };
        let l = &self.language;
        let mut out = vec![];
        let mut emit =
            |rule: &str, w: &Word, message: String, replacements: Vec<String>, severity: &str| {
                if o.checks == "spelling"
                    && !matches!(rule, "Danish.Spelling" | "Danish.UnknownWord")
                {
                    return;
                }
                if doc.ignored(rule, w.start) || o.allowed.contains(&w.lower) {
                    return;
                }
                out.push(Diagnostic::new(
                    &doc,
                    w,
                    rule,
                    severity,
                    message,
                    replacements
                        .iter()
                        .map(|r| match_case(&w.text, r))
                        .collect(),
                ));
            };
        for words in doc.segments() {
            let da_context = words.iter().any(|w| has(DA, &w.lower));
            for (i, w) in words.iter().enumerate() {
                let text = w.lower.as_str();
                let previous = i.checked_sub(1).map(|n| &words[n]);
                let following = words.get(i + 1);
                if let Some(prev) = previous
                    .filter(|p| p.lower == text && has(REPEATED, text) && doc.adjacent(p, w))
                {
                    let distinct = parsed.token(prev, &["VERB"]).is_some()
                        && parsed.token(w, &["ADP"]).is_some();
                    if !distinct {
                        emit(
                            "Danish.RepeatedWord",
                            w,
                            format!("'{}' står to gange i træk; kontrollér gentagelsen.", w.text),
                            vec![],
                            "warning",
                        );
                    }
                }
                if let Some(subject) = parsed
                    .token(w, &["PRON"])
                    .filter(|t| has(SUBJECTS, text) && t.deprel == "nsubj")
                {
                    let mut cursor = i + 1;
                    while cursor < words.len()
                        && has(ADVERBS, &words[cursor].lower)
                        && doc.adjacent(&words[cursor - 1], &words[cursor])
                    {
                        cursor += 1;
                    }
                    if let Some(verb) = words
                        .get(cursor)
                        .filter(|v| doc.adjacent(&words[cursor - 1], v))
                    {
                        if has(MODALS, &verb.lower)
                            && parsed.token(verb, &["AUX", "VERB"]).is_some()
                        {
                            cursor += 1;
                            while cursor < words.len()
                                && has(ADVERBS, &words[cursor].lower)
                                && doc.adjacent(&words[cursor - 1], &words[cursor])
                            {
                                cursor += 1;
                            }
                            if let Some(target) = words
                                .get(cursor)
                                .filter(|v| doc.adjacent(&words[cursor - 1], v))
                            {
                                let tags = l.analyses(&target.lower, "vb.");
                                if parsed.token(target, &["VERB", "AUX"]).is_some()
                                    && tags.iter().any(|a| a.tag.starts_with("vb.præs."))
                                    && !tags.iter().any(|a| a.tag.starts_with("vb.inf."))
                                {
                                    let forms = l.inflections(&target.lower, |t| t == "vb.inf.akt");
                                    if !forms.is_empty() {
                                        emit(
                                            "Danish.AfterModal",
                                            target,
                                            format!(
                                                "Efter '{}' forventes infinitiv; overvej '{}'.",
                                                verb.text, forms[0]
                                            ),
                                            forms,
                                            "warning",
                                        );
                                    }
                                }
                            }
                        } else {
                            let tags = l.analyses(&verb.lower, "vb.");
                            let ambiguous_de = text == "de"
                                && (!l.analyses(&verb.lower, "sb.").is_empty()
                                    || !l.analyses(&verb.lower, "adj.").is_empty());
                            if let Some(v) = parsed.token(verb, &[]).filter(|v| {
                                has("VERB AUX", &v.upostag)
                                    || l.analyses(&verb.lower, "sb.").is_empty()
                            }) {
                                let aux = parsed.child(v, &["aux", "aux:pass"])
                                    || previous.is_some_and(|p| has(MODALS, &p.lower));
                                if Parsed::linked(subject, v)
                                    && !aux
                                    && !ambiguous_de
                                    && tags.iter().any(|a| a.tag == "vb.inf.akt")
                                    && !tags
                                        .iter()
                                        .any(|a| a.tag.contains("præs") || a.tag.contains("præt"))
                                {
                                    let forms = l.inflections(&verb.lower, |t| t == "vb.præs.akt");
                                    if !forms.is_empty() {
                                        emit(
                                            "Danish.FiniteVerb",
                                            verb,
                                            format!(
                                                "Efter grundleddet '{}' forventes et bøjet udsagnsord; overvej '{}'.",
                                                w.text, forms[0]
                                            ),
                                            forms,
                                            "warning",
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some(next) =
                    following.filter(|n| has("en et den det", text) && doc.adjacent(w, n))
                {
                    let noun = parsed.token(
                        next,
                        if lexical_noun(l, &next.lower) {
                            &[]
                        } else {
                            &["NOUN"]
                        },
                    );
                    let linked = parsed
                        .token(w, &["DET"])
                        .zip(noun)
                        .is_some_and(|(a, n)| Parsed::linked(a, n));
                    let after = words.get(i + 2).filter(|a| doc.adjacent(next, a));
                    // A lexical noun followed by a finite verb also establishes a
                    // subject phrase when the tagger misreads an unfamiliar noun.
                    let subject_phrase = lexical_noun(l, &next.lower)
                        && after.is_some_and(|a| {
                            l.analyses(&a.lower, "vb.")
                                .iter()
                                .any(|v| v.tag.contains("præs") || v.tag.contains("præt"))
                        });
                    let modifier = after.is_some_and(|a| noun_modifier(l, &parsed, a));
                    let incomplete = incomplete_noun(l, source, next);
                    let dummy =
                        has("den det", text) && previous.is_some_and(|p| has(COPULA, &p.lower));
                    if (linked || subject_phrase)
                        && l.analyses(&next.lower, "adj.").is_empty()
                        && l.analyses(&next.lower, "pron").is_empty()
                        && l.analyses(&next.lower, "sb.")
                            .iter()
                            .any(|a| a.tag.contains(".ubest"))
                        && !modifier
                        && !incomplete
                        && !is_finite_verb(l, &next.lower)
                        && !dummy
                    {
                        if let Some((gender, number)) = l.noun_features(&next.lower) {
                            let expected = if has("en den", text) { "fk" } else { "itk" };
                            if number == "sg" && gender != expected {
                                let replace = match text {
                                    "en" => "et",
                                    "et" => "en",
                                    "den" => "det",
                                    _ => "den",
                                };
                                emit(
                                    "Danish.ArticleGender",
                                    w,
                                    format!(
                                        "'{}' har {gender}-køn i COR; overvej '{replace}'.",
                                        next.text
                                    ),
                                    vec![replace.into()],
                                    "warning",
                                );
                            }
                        }
                    }
                }
                if let Some(next) = following.filter(|n| doc.adjacent(w, n)) {
                    let noun = parsed.token(
                        next,
                        if lexical_noun(l, &next.lower) {
                            &[]
                        } else {
                            &["NOUN"]
                        },
                    );
                    let fragment = previous.is_none_or(|p| {
                        source[p.end..w.start].chars().any(|c| ",:;—–(".contains(c))
                    });
                    let linked = parsed.token(w, &[]).zip(noun).is_some_and(|(a, n)| {
                        !l.analyses(text, "adj.").is_empty()
                            && Parsed::linked(a, n)
                            && (a.deprel == "amod"
                                || (fragment
                                    && lexical_noun(l, &next.lower)
                                    && a.deprel == "advmod"))
                    });
                    let modifier = words
                        .get(i + 2)
                        .filter(|a| doc.adjacent(next, a))
                        .is_some_and(|a| noun_modifier(l, &parsed, a));
                    // A potential adverb immediately after the verb (or inverted
                    // subject) is too ambiguous for a noun-agreement warning.
                    let after_verb = !fragment
                        && (previous.is_some_and(|p| parsed.token(p, &["VERB", "AUX"]).is_some())
                            || (i >= 2
                                && previous.is_some_and(|p| parsed.token(p, &["PRON"]).is_some())
                                && parsed.token(&words[i - 2], &["VERB", "AUX"]).is_some()));
                    let adverb_context = after_verb && text.ends_with('t');
                    // A directly attached singular determiner disambiguates invariant
                    // nouns (et svar) without trusting the parser's predicted gender.
                    let singular_determiner = previous
                        .filter(|p| has("en et", &p.lower) && doc.adjacent(p, w))
                        .and_then(|p| parsed.token(p, &["DET"]))
                        .zip(noun)
                        .is_some_and(|(d, n)| Parsed::linked(d, n));
                    if linked
                        && !modifier
                        && !adverb_context
                        && !incomplete_noun(l, source, next)
                        && !is_finite_verb(l, &next.lower)
                        && l.analyses(&next.lower, "pron").is_empty()
                        && (singular_determiner || l.analyses(&next.lower, "adj.").is_empty())
                    {
                        if let Some((gender, number)) =
                            l.noun_features_number(&next.lower, singular_determiner.then_some("sg"))
                        {
                            let definite_noun = l
                                .analyses(&next.lower, "sb.")
                                .iter()
                                .filter(|a| !a.tag.contains(".gen"))
                                .all(|a| a.tag.contains(".best"));
                            let mut explicit = false;
                            let mut cursor = i;
                            while cursor > 0 && cursor > i.saturating_sub(4) {
                                cursor -= 1;
                                let prefix = &words[cursor];
                                let gap = &source[prefix.end..words[cursor + 1].start];
                                if !gap.chars().all(|c| " \t*\"“”«»".contains(c))
                                    || has("slags type form art", &prefix.lower)
                                {
                                    break;
                                }
                                let possessive = prefix.lower.ends_with("'s")
                                    || prefix.lower.ends_with("’s")
                                    || l.analyses(&prefix.lower, "sb.")
                                        .iter()
                                        .any(|a| a.tag.contains(".gen"))
                                    || (prefix.lower.ends_with('s')
                                        && parsed.token(prefix, &["NOUN", "PROPN"]).is_some());
                                let attached_determiner = has(DEFINITE, &prefix.lower)
                                    && parsed.token(prefix, &[]).zip(noun).is_some_and(|(d, n)| {
                                        Parsed::linked(d, n) && has("det nmod:poss", &d.deprel)
                                    });
                                if attached_determiner || possessive {
                                    explicit = true;
                                    break;
                                }
                                if !(has("mest mindst her der første anden", &prefix.lower)
                                    || !l.analyses(&prefix.lower, "adj.").is_empty()
                                    || parsed.token(prefix, &["ADV"]).is_some())
                                {
                                    break;
                                }
                            }
                            let inverted = previous.is_some_and(|p| has("den det de", &p.lower))
                                && i >= 2
                                && has(COPULA, &words[i - 2].lower);
                            let prior_adj =
                                previous.is_some_and(|p| parsed.token(p, &["ADJ"]).is_some());
                            let definite = definite_noun || explicit;
                            let imperative =
                                l.analyses(text, "vb.").iter().any(|a| a.tag == "vb.imp");
                            let inflected_fragment = fragment
                                && text.ends_with('t')
                                && parsed.token(w, &["ADV"]).is_some();
                            let lexical_adverb =
                                !l.analyses(text, "adv").is_empty() && !text.ends_with('t');
                            if !definite_noun
                                && (!lexical_adverb || explicit || singular_determiner)
                                && (!imperative
                                    || explicit
                                    || singular_determiner
                                    || inflected_fragment)
                                && (!inverted
                                    || self
                                        .adjective_target(text, &gender, &number, true)
                                        .is_empty())
                                && (definite
                                    || !(prior_adj
                                        || noun.is_some_and(|n| parsed.child(n, &["nmod:poss"]))))
                            {
                                let forms = self.adjective_target(text, &gender, &number, definite);
                                if !forms.is_empty() {
                                    emit(
                                        "Danish.AdjectiveAgreement",
                                        w,
                                        format!(
                                            "Bøjningen skal passe til '{}'; overvej '{}'.",
                                            next.text, forms[0]
                                        ),
                                        forms,
                                        "warning",
                                    );
                                }
                            }
                        }
                    }
                }
                if following.is_some_and(|n| has(COPULA, &n.lower) && doc.adjacent(w, n)) {
                    let features = if has("vi de", text) {
                        Some(("fk".into(), "pl".into()))
                    } else if has("jeg du han hun", text) {
                        Some(("fk".into(), "sg".into()))
                    } else {
                        l.noun_features(text)
                    };
                    let mut cursor = i + 2;
                    while cursor < words.len()
                        && has(ADVERBS, &words[cursor].lower)
                        && doc.adjacent(&words[cursor - 1], &words[cursor])
                    {
                        cursor += 1;
                    }
                    if let (Some((gender, number)), Some(adj)) = (
                        features,
                        words
                            .get(cursor)
                            .filter(|a| doc.adjacent(&words[cursor - 1], a)),
                    ) {
                        let after = words.get(cursor + 1);
                        let noun_phrase = after.is_some_and(|n| {
                            doc.adjacent(adj, n) && !l.analyses(&n.lower, "sb.").is_empty()
                        });
                        let subj = parsed.token(w, &["NOUN", "PRON"]);
                        let linked = subj
                            .zip(parsed.token(adj, &["ADJ"]))
                            .is_some_and(|(s, a)| s.deprel == "nsubj" && Parsed::linked(s, a));
                        let idiom = adj.lower == "vant"
                            || (adj.lower == "klar" && after.is_some_and(|a| a.lower == "over"));
                        let participle = !l.analyses(&adj.lower, "vb.").is_empty();
                        let prep = subj.is_some_and(|s| parsed.child(s, &["case"]))
                            || words[i.saturating_sub(3)..i]
                                .iter()
                                .any(|w| has("at for af med til om", &w.lower));
                        let definite = has(SUBJECTS, text)
                            || number == "pl"
                            || l.analyses(text, "sb.")
                                .iter()
                                .any(|a| a.tag.contains(".best"));
                        if linked && definite && !noun_phrase && !idiom && !participle && !prep {
                            let forms = self.adjective_target(&adj.lower, &gender, &number, false);
                            if !forms.is_empty() {
                                emit(
                                    "Danish.PredicateAgreement",
                                    adj,
                                    format!(
                                        "Tillægsordet skal passe til '{}'; overvej '{}'.",
                                        w.text, forms[0]
                                    ),
                                    forms,
                                    "warning",
                                );
                            }
                        }
                    }
                }
                let len = text.chars().count();
                if o.checks == "grammar"
                    || !da_context
                    || o.allowed.contains(text)
                    || !(4..=40).contains(&len)
                    || !w.text.chars().all(char::is_alphabetic)
                    || w.text.chars().any(char::is_uppercase)
                    || l.spelled(&w.text)
                    || l.english(&w.text)
                {
                    continue;
                }
                let mut productive = ["u", "un"].iter().any(|p| {
                    text.strip_prefix(p)
                        .is_some_and(|rest| rest.chars().count() > 3 && l.spelled(rest))
                });
                for suffix in [
                    "ende", "erne", "ede", "ene", "ens", "er", "et", "en", "es", "e", "s",
                ] {
                    if let Some(stem) = text.strip_suffix(suffix).filter(|s| s.chars().count() > 3)
                    {
                        productive |= l.english(stem);
                        let chars: Vec<_> = stem.chars().collect();
                        if chars.len() > 1 && chars[chars.len() - 1] == chars[chars.len() - 2] {
                            productive |=
                                l.english(&chars[..chars.len() - 1].iter().collect::<String>());
                        }
                    }
                }
                let candidates: Vec<_> = l
                    .suggestions(&w.text)
                    .into_iter()
                    .filter(|s| s.chars().all(char::is_alphabetic) && s.to_lowercase() != text)
                    .map(|s| s.to_lowercase())
                    .collect();
                let near: Vec<_> = candidates
                    .iter()
                    .filter(|s| {
                        l.forms.contains_key(*s) && distance(text, s) <= if len < 7 { 1 } else { 2 }
                    })
                    .cloned()
                    .collect();
                let first: String = text.chars().take(2).collect();
                let mut choices: Vec<String> = if productive {
                    vec![]
                } else {
                    l.known
                        .get(text)
                        .into_iter()
                        .flatten()
                        .filter(|s| s.starts_with(&first))
                        .cloned()
                        .collect()
                };
                if choices.is_empty() && !productive && len >= 7 {
                    let min = near.iter().map(|s| distance(text, s)).min().unwrap_or(0);
                    choices = near
                        .iter()
                        .filter(|s| {
                            l.forms.contains_key(*s)
                                && s.chars().take(4).eq(text.chars().take(4))
                                && s.chars().rev().take(3).eq(text.chars().rev().take(3))
                                && internal_edit(text, s, false)
                                && distance(text, s) == min
                        })
                        .cloned()
                        .collect();
                }
                choices.sort();
                choices.dedup();
                if choices.len() == 1 {
                    let candidate = &choices[0];
                    let from_search_help = l
                        .known
                        .get(text)
                        .is_some_and(|forms| forms.contains(candidate));
                    // A two-letter stem deletion can invent an unrelated noun.
                    // Require inflectional evidence or a dictionary suggestion;
                    // duplicated chunks are handled separately.
                    let inflectional = !l.analyses(candidate, "vb.").is_empty()
                        || !l.analyses(candidate, "adj.").is_empty();
                    if !from_search_help
                        && !inflectional
                        && distance(text, candidate) > 1
                        && !internal_edit(text, candidate, true)
                        && !l.confirms_suggestion(text, candidate)
                    {
                        choices.clear();
                    }
                }
                if choices.len() == 1 {
                    emit(
                        "Danish.Spelling",
                        w,
                        format!("Ordet genkendes ikke; overvej '{}'.", choices[0]),
                        choices,
                        "warning",
                    );
                } else if o.uncertain {
                    emit(
                        "Danish.UnknownWord",
                        w,
                        "Ukendt ord; kan være et navn, fagord eller en stavefejl.".into(),
                        candidates.into_iter().take(5).collect(),
                        "suggestion",
                    );
                }
            }
        }
        // Exact, sourced rules take precedence, including their local suppression.
        for rule in self
            .rules
            .rules
            .iter()
            .filter(|r| r.record.category == "spelling")
        {
            out.retain(|d| {
                !(d.check == "Danish.Spelling"
                    && rule.patterns.iter().any(|(p, _)| p.is_match(&d.r#match)))
            });
        }
        out.extend(self.rules.check(&doc, &o.profile, &o.checks, &o.allowed));
        out.sort_by_key(|d| (d.start, d.check.clone()));
        Ok(out)
    }
}
pub fn distance(a: &str, b: &str) -> usize {
    let b: Vec<_> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, a) in a.chars().enumerate() {
        let mut next = vec![i + 1];
        for (j, b) in b.iter().enumerate() {
            next.push(
                (next[j] + 1)
                    .min(row[j + 1] + 1)
                    .min(row[j] + usize::from(a != *b)),
            );
        }
        row = next;
    }
    row[b.len()]
}
fn internal_edit(a: &str, b: &str, require_repeated: bool) -> bool {
    let a: Vec<_> = a.chars().collect();
    let b: Vec<_> = b.chars().collect();
    let (short, long) = if a.len() < b.len() { (a, b) } else { (b, a) };
    let delta = long.len() - short.len();
    matches!(delta, 1 | 2)
        && (2..long.len() - delta - 1).any(|i| {
            let repeated = !require_repeated
                || delta == 1
                || long[i - delta..i] == long[i..i + delta]
                || (i + 2 * delta <= long.len()
                    && long[i..i + delta] == long[i + delta..i + 2 * delta]);
            repeated && long[..i].iter().chain(&long[i + delta..]).eq(short.iter())
        })
}
