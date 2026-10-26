use anyhow::{Context, Result};
use spellbook::Dictionary;
use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap},
    fs,
    path::Path,
};
#[derive(Debug, Clone)]
pub struct Analysis {
    pub lemma: u32,
    pub tag: String,
}
pub struct Language {
    pub forms: HashMap<String, Vec<Analysis>>,
    lemmas: HashMap<u32, Vec<(String, String)>>,
    pub known: HashMap<String, BTreeSet<String>>,
    da: Dictionary,
    en: Dictionary,
    suggestion_cache: RefCell<HashMap<String, Vec<String>>>,
    confirmation_cache: RefCell<HashMap<(String, String), bool>>,
}
impl Language {
    pub fn load(dir: &Path) -> Result<Self> {
        let read = |name: &str| {
            fs::read_to_string(dir.join(name)).with_context(|| {
                format!(
                    "Sprogdata mangler: {}. Kør `danish setup`.",
                    dir.join(name).display()
                )
            })
        };
        let da = Dictionary::new(
            &read("da_DK.aff")?,
            &normalize_numeric_dictionary(&read("da_DK.dic")?),
        )
        .map_err(|e| anyhow::anyhow!("Dansk Hunspell-ordbog: {e}"))?;
        let en = Dictionary::new(&read("en_US.aff")?, &read("en_US.dic")?)
            .map_err(|e| anyhow::anyhow!("Engelsk hjælpeordbog: {e}"))?;
        let mut forms: HashMap<String, Vec<Analysis>> = HashMap::new();
        let mut lemmas: HashMap<u32, Vec<(String, String)>> = HashMap::new();
        for line in read("cor1.5.1.0.tsv")?.lines() {
            let f: Vec<_> = line.split('\t').collect();
            if f.len() != 6 || !matches!(f[5], "N" | "K") {
                continue;
            }
            let lemma = f[0].split('.').nth(1).unwrap_or("0").parse::<u32>()?;
            let tag = f[3].to_owned();
            let form = f[4].to_lowercase();
            forms.entry(form.clone()).or_default().push(Analysis {
                lemma,
                tag: tag.clone(),
            });
            lemmas.entry(lemma).or_default().push((form, tag));
        }
        let mut known: HashMap<String, BTreeSet<String>> = HashMap::new();
        for line in read("corsoegehjaelp1.0.tsv")?.lines() {
            let f: Vec<_> = line.split('\t').collect();
            if f.len() == 4
                && f[3].chars().all(char::is_alphabetic)
                && f[1].split_whitespace().all(|s| {
                    s.chars().all(char::is_alphabetic) && forms.contains_key(&s.to_lowercase())
                })
            {
                known
                    .entry(f[3].to_lowercase())
                    .or_default()
                    .insert(f[1].to_lowercase());
            }
        }
        Ok(Self {
            forms,
            lemmas,
            known,
            da,
            en,
            suggestion_cache: RefCell::new(HashMap::new()),
            confirmation_cache: RefCell::new(HashMap::new()),
        })
    }
    pub fn analyses(&self, word: &str, prefix: &str) -> Vec<&Analysis> {
        self.forms
            .get(word)
            .into_iter()
            .flatten()
            .filter(|a| a.tag.starts_with(prefix))
            .collect()
    }
    pub fn inflections(&self, word: &str, predicate: impl Fn(&str) -> bool) -> Vec<String> {
        self.analyses(word, "")
            .iter()
            .flat_map(|a| self.lemmas.get(&a.lemma).into_iter().flatten())
            .filter(|(_, t)| predicate(t))
            .map(|(w, _)| w.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
    pub fn spelled(&self, w: &str) -> bool {
        self.forms.contains_key(&w.to_lowercase()) || self.da.check(w)
    }
    pub fn english(&self, w: &str) -> bool {
        self.en.check(w)
    }
    pub fn suggestions(&self, w: &str) -> Vec<String> {
        if let Some(result) = self.suggestion_cache.borrow().get(w) {
            return result.clone();
        }
        // We only need close COR forms, not a full search through productive
        // Hunspell compounds. The latter can dominate runtime on technical prose.
        let mut out = vec![];
        let chars: Vec<_> = w.chars().collect();
        let alphabet: Vec<_> = "abcdefghijklmnopqrstuvwxyzæøåéüäö".chars().collect();
        let mut consider = |candidate: String| {
            if self.forms.contains_key(&candidate) {
                out.push(candidate);
            }
        };
        for i in 0..chars.len() {
            consider(chars[..i].iter().chain(&chars[i + 1..]).collect());
            for &letter in &alphabet {
                consider(
                    chars[..i]
                        .iter()
                        .copied()
                        .chain([letter])
                        .chain(chars[i + 1..].iter().copied())
                        .collect(),
                );
            }
        }
        for i in 0..=chars.len() {
            for &letter in &alphabet {
                consider(
                    chars[..i]
                        .iter()
                        .copied()
                        .chain([letter])
                        .chain(chars[i..].iter().copied())
                        .collect(),
                );
            }
        }
        // Two internal insertions/deletions only qualify when the four-character
        // prefix and three-character suffix survive, matching the warning rule.
        if chars.len() >= 7 {
            for i in 4..=chars.len() - 3 {
                if i + 2 <= chars.len() - 3 {
                    consider(chars[..i].iter().chain(&chars[i + 2..]).collect());
                }
                for &a in &alphabet {
                    for &b in &alphabet {
                        consider(
                            chars[..i]
                                .iter()
                                .copied()
                                .chain([a, b])
                                .chain(chars[i..].iter().copied())
                                .collect(),
                        );
                    }
                }
            }
        }
        out.sort_by_key(|s| {
            (
                crate::checker::distance(w, s),
                !self.forms.contains_key(&s.to_lowercase()),
                s.clone(),
            )
        });
        out.dedup();
        out.truncate(32);
        // Repeated technical words across files should not repeat expensive searches.
        let mut cache = self.suggestion_cache.borrow_mut();
        if cache.len() >= 4096 {
            cache.clear();
        }
        cache.insert(w.to_owned(), out.clone());
        out
    }
    /// Full compound suggestion search is reserved for otherwise plausible
    /// two-character corrections; never run it for every unknown technical word.
    pub fn confirms_suggestion(&self, word: &str, candidate: &str) -> bool {
        let key = (word.to_owned(), candidate.to_owned());
        if let Some(result) = self.confirmation_cache.borrow().get(&key) {
            return *result;
        }
        let mut suggestions = vec![];
        self.da.suggest(word, &mut suggestions);
        let result = suggestions.iter().any(|s| s.to_lowercase() == candidate);
        let mut cache = self.confirmation_cache.borrow_mut();
        if cache.len() >= 4096 {
            cache.clear();
        }
        cache.insert(key, result);
        result
    }
    pub fn noun_features(&self, w: &str) -> Option<(String, String)> {
        self.noun_features_number(w, None)
    }
    pub fn noun_features_number(&self, w: &str, number: Option<&str>) -> Option<(String, String)> {
        let features: BTreeSet<_> = self
            .analyses(w, "sb.")
            .iter()
            .filter(|a| !a.tag.contains(".gen") && !a.tag.contains(".sms"))
            .map(|a| {
                (
                    a.tag.split('.').nth(1).unwrap().to_owned(),
                    if a.tag.contains(".pl.") { "pl" } else { "sg" }.to_owned(),
                )
            })
            .filter(|(_, n)| number.is_none_or(|wanted| n == wanted))
            .collect();
        if features.len() == 1 {
            features.into_iter().next()
        } else {
            None
        }
    }
}

/// Preserve literal slashes in legacy entries (e.g. quoted A/S) that are not numeric flag lists.
/// Original downloaded data stays untouched; only the parser input is normalized.
pub fn normalize_numeric_dictionary(input: &str) -> String {
    input
        .lines()
        .map(|line| {
            if let Some((_, flags)) = line.split_once('/') {
                let flags = flags.split_whitespace().next().unwrap_or("");
                if !flags.is_empty()
                    && !flags.split(',').all(|s| s.parse::<u16>().is_ok())
                    && !line.contains("\\/")
                {
                    return line.replace('/', "\\/");
                }
            }
            line.to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}
