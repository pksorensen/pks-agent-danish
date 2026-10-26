use crate::{
    diagnostic::{Diagnostic, match_case},
    prose::{Prose, Word},
};
use anyhow::{Context, Result, bail};
use include_dir::{Dir, include_dir};
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};
static STYLES: Dir = include_dir!("$CARGO_MANIFEST_DIR/styles");
static RECORDS: Dir = include_dir!("$CARGO_MANIFEST_DIR/rules");
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Case {
    pub text: String,
    pub matches: Vec<String>,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Record {
    pub id: String,
    pub category: String,
    pub status: String,
    pub summary: String,
    pub explanation: String,
    pub source: String,
    pub source_note: String,
    pub exceptions: Vec<String>,
    pub autofix: bool,
    pub cases: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Spec {
    extends: String,
    message: String,
    level: String,
    #[serde(default)]
    ignorecase: bool,
    #[serde(default)]
    tokens: Vec<String>,
    #[serde(default)]
    swap: HashMap<String, String>,
}
pub struct Rule {
    pub record: Record,
    pub patterns: Vec<(Regex, Option<String>)>,
    message: String,
    level: String,
}
pub struct Rules {
    pub rules: Vec<Rule>,
}
fn compile(record: Record, yaml: &str) -> Result<Rule> {
    let spec: Spec = serde_yaml::from_str(yaml).with_context(|| record.id.clone())?;
    if !matches!(spec.extends.as_str(), "existence" | "substitution") {
        bail!("{}: kun existence og substitution understøttes", record.id);
    }
    if !matches!(spec.level.as_str(), "error" | "warning" | "suggestion") {
        bail!("Ugyldigt niveau i {}", record.id);
    }
    let mut patterns = vec![];
    let entries = if spec.extends == "existence" {
        spec.tokens
            .into_iter()
            .map(|t| (t, None))
            .collect::<Vec<_>>()
    } else {
        spec.swap.into_iter().map(|(p, s)| (p, Some(s))).collect()
    };
    for (pattern, replacement) in entries {
        patterns.push((
            RegexBuilder::new(&format!(r"\b(?:{pattern})\b"))
                .case_insensitive(spec.ignorecase)
                .build()?,
            replacement,
        ));
    }
    patterns.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
    Ok(Rule {
        record,
        patterns,
        message: spec.message,
        level: spec.level,
    })
}
impl Rules {
    pub fn test_examples(&self) -> Result<usize> {
        let mut count = 0;
        for rule in &self.rules {
            if rule.record.cases.is_empty() {
                bail!("{} mangler eksempler", rule.record.id);
            }
            for case in &rule.record.cases {
                let doc = Prose::new(&case.text, true);
                let actual = rule.check(&doc, &HashSet::new());
                let matches: Vec<_> = actual.iter().map(|a| &a.r#match).collect();
                if matches != case.matches.iter().collect::<Vec<_>>() {
                    bail!(
                        "{}: {:?}: forventede {:?}, fik {:?}",
                        rule.record.id,
                        case.text,
                        case.matches,
                        matches
                    );
                }
                count += 1;
            }
        }
        Ok(count)
    }
    pub fn load(custom: Option<&Path>) -> Result<Self> {
        let mut rules = vec![];
        for file in RECORDS.files() {
            let record: Record = serde_json::from_slice(file.contents())?;
            let (pack, name) = record
                .id
                .split_once('.')
                .context("Regel-id mangler pakke")?;
            let path = format!("{pack}/{name}.yml");
            let yaml = STYLES
                .get_file(&path)
                .context("Reglens YAML mangler")?
                .contents_utf8()
                .context("YAML UTF-8")?;
            rules.push(compile(record, yaml)?);
        }
        if let Some(root) = custom {
            let directory = root.join("rules");
            let mut external_ids = HashSet::new();
            let valid_id = Regex::new(r"^[A-Za-z]+\.[A-Z][A-Za-z0-9]+$")?;
            for entry in fs::read_dir(directory)? {
                let path = entry?.path();
                if path.extension().is_some_and(|s| s == "json") {
                    let r: Record = serde_json::from_str(&fs::read_to_string(path)?)?;
                    if !valid_id.is_match(&r.id) {
                        bail!("Ugyldigt regel-id: {}", r.id);
                    }
                    if !external_ids.insert(r.id.clone()) {
                        bail!("Dubleret regel-id: {}", r.id);
                    }
                    let (pack, name) = r.id.split_once('.').context("Ugyldigt regel-id")?;
                    let yaml = fs::read_to_string(
                        root.join("styles").join(pack).join(format!("{name}.yml")),
                    )?;
                    let compiled = compile(r, &yaml)?;
                    if let Some(index) =
                        rules.iter().position(|x| x.record.id == compiled.record.id)
                    {
                        rules[index] = compiled;
                    } else {
                        rules.push(compiled);
                    }
                }
            }
        }
        rules.sort_by(|a, b| a.record.id.cmp(&b.record.id));
        Ok(Self { rules })
    }
    pub fn check(
        &self,
        doc: &Prose<'_>,
        profile: &str,
        checks: &str,
        allowed: &HashSet<String>,
    ) -> Vec<Diagnostic> {
        self.rules
            .iter()
            .filter(|r| {
                active(&r.record.id, profile)
                    && checks != "grammar"
                    && (checks != "spelling" || r.record.category == "spelling")
            })
            .flat_map(|r| r.check(doc, allowed))
            .collect()
    }
}
fn active(id: &str, profile: &str) -> bool {
    if profile == "experimental" {
        id.starts_with("Experimental.")
    } else {
        id.starts_with("Dansk.")
            || (profile != "dansk" && id.starts_with("KlartSprog."))
            || (profile == "agentics" && id.starts_with("Agentics."))
    }
}
impl Rule {
    pub fn check(&self, doc: &Prose<'_>, allowed: &HashSet<String>) -> Vec<Diagnostic> {
        let mut out = vec![];
        for words in doc.segments() {
            let start = words[0].start;
            let end = words.last().unwrap().end;
            for (pattern, replacement) in &self.patterns {
                for m in pattern.find_iter(&doc.masked[start..end]) {
                    let word = Word {
                        text: m.as_str().to_owned(),
                        lower: m.as_str().to_lowercase(),
                        start: start + m.start(),
                        end: start + m.end(),
                    };
                    if doc.ignored(&self.record.id, word.start) || allowed.contains(&word.lower) {
                        continue;
                    }
                    let suggestions = replacement
                        .as_ref()
                        .map(|r| vec![match_case(&word.text, r)])
                        .unwrap_or_default();
                    let message = if let Some(replace) = suggestions.first() {
                        self.message
                            .replacen("%s", replace, 1)
                            .replacen("%s", &word.text, 1)
                    } else {
                        self.message.replace("%s", &word.text)
                    };
                    let mut diagnostic = Diagnostic::new(
                        doc,
                        &word,
                        &self.record.id,
                        &self.level,
                        message,
                        suggestions,
                    );
                    diagnostic.source.clone_from(&self.record.source);
                    out.push(diagnostic);
                }
            }
        }
        out.sort_by_key(|d| (d.start, d.check.clone()));
        out
    }
}
pub fn new_rule(id: &str, root: &Path) -> Result<()> {
    if !Regex::new(r"^Experimental\.[A-Z][A-Za-z0-9]+$")
        .unwrap()
        .is_match(id)
    {
        bail!("Nye regler starter som Experimental.Regelnavn");
    }
    let name = id.split_once('.').unwrap().1;
    let yaml = root.join("styles/Experimental").join(format!("{name}.yml"));
    let record = root.join("rules").join(format!("{id}.json"));
    if yaml.exists() || record.exists() {
        bail!("Reglen findes allerede: {id}");
    }
    let r = Record {
        id: id.into(),
        category: "style".into(),
        status: "experimental".into(),
        summary: "Udfyld reglens formål".into(),
        explanation: "Udfyld forklaring og afgrænsning.".into(),
        source: "TODO".into(),
        source_note: "Angiv en kilde eller begrundet stilbeslutning.".into(),
        exceptions: vec!["Tilføj kendte undtagelser.".into()],
        autofix: false,
        cases: vec![
            Case {
                text: "Her er EKSEMPELMOENSTER.".into(),
                matches: vec!["EKSEMPELMOENSTER".into()],
            },
            Case {
                text: "Her er en korrekt formulering.".into(),
                matches: vec![],
            },
            Case {
                text: "Skriv `EKSEMPELMOENSTER` i koden.".into(),
                matches: vec![],
            },
        ],
    };
    fs::create_dir_all(yaml.parent().unwrap())?;
    fs::create_dir_all(record.parent().unwrap())?;
    fs::write(
        &yaml,
        "extends: existence\nmessage: \"Udfyld en hjælpsom forklaring til '%s'.\"\nlevel: warning\nignorecase: true\ntokens:\n  - EKSEMPELMOENSTER\n",
    )?;
    fs::write(&record, serde_json::to_string_pretty(&r)? + "\n")?;
    println!(
        "Oprettet {id}. Brug --rules-dir {} ved lint.",
        root.display()
    );
    Ok(())
}
