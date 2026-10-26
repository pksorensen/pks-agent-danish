use anyhow::{Context, Result, bail};
use std::{
    cell::OnceCell,
    collections::HashMap,
    path::{Path, PathBuf},
};
use udpipe_rs::{Model, Word};
pub struct Parser {
    model: OnceCell<Result<Model, String>>,
    path: Option<PathBuf>,
}
pub struct Parsed {
    tokens: HashMap<usize, Word>,
}
impl Parser {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        if let Some(path) = path {
            std::fs::metadata(path).context("Dansk DDT-analysemodel")?;
        }
        Ok(Self {
            model: OnceCell::new(),
            path: path.map(Path::to_path_buf),
        })
    }
    pub fn parse(&self, text: &str) -> Result<Parsed> {
        let model = self
            .model
            .get_or_init(|| {
                match &self.path {
                    Some(path) => Model::load(path),
                    None => Model::load_from_memory(include_bytes!("../models/danish-ddt.udpipe")),
                }
                .map_err(|e| e.to_string())
            })
            .as_ref()
            .map_err(|e| anyhow::anyhow!("Dansk DDT-analysemodel: {e}"))?;
        let mut tokens = HashMap::new();
        let mut cursor = 0;
        for token in model.parse(text)? {
            let Some(delta) = text[cursor..].find(&token.form) else {
                bail!(
                    "Analysemodellen ændrede en ordform; kan ikke placere {:?} i originalteksten",
                    token.form
                );
            };
            if text[cursor..cursor + delta]
                .chars()
                .any(char::is_alphabetic)
            {
                bail!(
                    "Analysemodellen sprang tekst over; rapporten afbrydes for at bevare korrekte positioner"
                );
            }
            let start = cursor + delta;
            cursor = start + token.form.len();
            tokens.insert(start, token);
        }
        Ok(Parsed { tokens })
    }
}
impl Parsed {
    pub fn empty() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }
    pub fn token(&self, w: &crate::prose::Word, parts: &[&str]) -> Option<&Word> {
        self.tokens.get(&w.start).filter(|t| {
            t.form == w.text && (parts.is_empty() || parts.contains(&t.upostag.as_str()))
        })
    }
    pub fn linked(a: &Word, b: &Word) -> bool {
        a.sentence_id == b.sentence_id && a.head == b.id
    }
    pub fn child(&self, t: &Word, rels: &[&str]) -> bool {
        self.tokens
            .values()
            .any(|c| Self::linked(c, t) && rels.contains(&c.deprel.as_str()))
    }
}
