use crate::prose::{Prose, Word};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Diagnostic {
    pub check: String,
    pub severity: String,
    pub message: String,
    pub line: usize,
    pub span: [usize; 2],
    pub r#match: String,
    pub suggestions: Vec<String>,
    pub source: String,
    pub start: usize,
    pub end: usize,
}
impl Diagnostic {
    pub fn new(
        doc: &Prose<'_>,
        word: &Word,
        rule: &str,
        severity: &str,
        message: String,
        suggestions: Vec<String>,
    ) -> Self {
        let (line, span) = doc.location(word.start, word.end);
        Self {
            check: rule.to_owned(),
            severity: severity.to_owned(),
            message,
            line,
            span,
            r#match: word.text.clone(),
            suggestions,
            source: "COR / Stavekontrolden / DDT".to_owned(),
            start: doc.source[..word.start].chars().count(),
            end: doc.source[..word.end].chars().count(),
        }
    }
}
pub fn match_case(original: &str, replacement: &str) -> String {
    if original
        .chars()
        .filter(|c| c.is_alphabetic())
        .all(char::is_uppercase)
    {
        replacement.to_uppercase()
    } else if original.chars().next().is_some_and(char::is_uppercase) {
        let mut chars = replacement.chars();
        format!(
            "{}{}",
            chars
                .next()
                .map(|c| c.to_uppercase().to_string())
                .unwrap_or_default(),
            chars.as_str()
        )
    } else {
        replacement.to_owned()
    }
}
