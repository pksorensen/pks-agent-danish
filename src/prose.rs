//! Offset-preserving extraction. Internal offsets are UTF-8 bytes; public spans are Unicode characters.
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;

pub const DA: &str = "jeg du han hun vi de det den der dem en et og men ikke som til af på med fra har havde er var blev bliver skal skulle vil ville kan kunne må måtte også fordi hvis når hvor hvad vores deres sin sine sit noget meget denne dette disse være have at mig dig sig for om så selv hele ved mere alle man jo få først nu efter før hvordan hvorfor min mit din dit uden over under ind ud eller igen sammen sådan både mellem imod gennem hver hvilket faktisk kun end her dem os jer i stedet flere andet andre hverken ingen endnu altid mest dog nok helt lige netop samtidig derfor blevet gør gjort gøre samme næsten dag godt gang ting måde mennesker år";
const EN: &str = "the this that these those with from and but not are was were been being is will would can could should have has had which what when where why how your their its our you they we my of to for as it in on into without about after before than then there here just do does did don't doesn't isn't wasn't can't won't let let's i'll i've that's it's so also all any some only now one each both more most very while through between because if else by an a";
pub fn has(set: &str, value: &str) -> bool {
    set.split_whitespace().any(|s| s == value)
}
#[derive(Clone, Debug)]
pub struct Word {
    pub text: String,
    pub lower: String,
    pub start: usize,
    pub end: usize,
}
pub struct Prose<'a> {
    pub source: &'a str,
    pub masked: String,
    ignored: Vec<(String, usize, usize)>,
}
fn mask(bytes: &mut [u8], start: usize, end: usize) {
    for b in &mut bytes[start..end] {
        if !matches!(*b, b'\n' | b'\r') {
            *b = b' ';
        }
    }
}
fn mask_re(bytes: &mut [u8], expression: &str, skip: usize) {
    let snapshot =
        String::from_utf8(bytes.to_vec()).expect("only complete UTF-8 ranges are masked");
    for m in Regex::new(expression)
        .expect("internal regex")
        .find_iter(&snapshot)
    {
        mask(bytes, m.start() + skip, m.end());
    }
}
impl<'a> Prose<'a> {
    pub fn new(source: &'a str, markdown: bool) -> Self {
        let mut bytes = source.as_bytes().to_vec();
        let mut ignored = vec![];
        let mut active = HashMap::new();
        static SUPPRESS: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"<!--\s*(?:vale|danish)\s+([\w.]+)\s*=\s*(NO|YES)\s*-->").unwrap()
        });
        for c in SUPPRESS.captures_iter(source) {
            let m = c.get(0).unwrap();
            let name = c[1].to_owned();
            if &c[2] == "NO" {
                active.entry(name).or_insert(m.end());
            } else if let Some(start) = active.remove(&name) {
                ignored.push((name, start, m.start()));
            }
        }
        for (name, start) in active {
            ignored.push((name, start, source.len()));
        }
        if markdown {
            let front = source.lines().next().is_some_and(|s| s.trim() == "---");
            let mut frontmatter = front;
            let mut fence: Option<(u8, usize)> = None;
            let mut offset = 0;
            let metadata = Regex::new(r#"^(title|subtitle|description):\s*["']?"#).unwrap();
            let open = Regex::new(r"^ {0,3}(`{3,}|~{3,})(.*)$").unwrap();
            let reference = Regex::new(r"^ {0,3}\[[^]]+\]:").unwrap();
            for (index, line) in source.split_inclusive('\n').enumerate() {
                let end = offset + line.len();
                let content = line.trim_end_matches(['\r', '\n']);
                if frontmatter {
                    if let Some(m) = metadata.find(line) {
                        mask(&mut bytes, offset, offset + m.end());
                    } else {
                        mask(&mut bytes, offset, end);
                    }
                    if index > 0 && matches!(line.trim(), "---" | "...") {
                        frontmatter = false;
                    }
                } else if let Some((delimiter, length)) = fence {
                    mask(&mut bytes, offset, end);
                    let leading = content.len() - content.trim_start_matches(' ').len();
                    let rest = content.trim();
                    if leading <= 3 && rest.len() >= length && rest.bytes().all(|b| b == delimiter)
                    {
                        fence = None;
                    }
                } else if let Some(c) = open.captures(content) {
                    let d = &c[1];
                    if !d.starts_with('`') || !c[2].contains('`') {
                        fence = Some((d.as_bytes()[0], d.len()));
                        mask(&mut bytes, offset, end);
                    }
                } else if line.starts_with("    ")
                    || line.starts_with('\t')
                    || reference.is_match(line)
                {
                    mask(&mut bytes, offset, end);
                }
                offset = end;
            }
            mask_re(
                &mut bytes,
                r"(?is)<!--.*?(?:-->|\z)|<(?:pre|code|script|style)\b[^>]*>.*?</(?:pre|code|script|style)\s*>|<[^>\n]+>",
                0,
            );
            // Backtick delimiter lengths must match; bounded backtracking is only used for Markdown syntax.
            let snapshot = String::from_utf8(bytes.clone()).unwrap();
            let inline =
                fancy_regex::Regex::new(r"(?<!`)(`+)(?!`)([\s\S]*?)(?<!`)\1(?!`)").unwrap();
            for m in inline.find_iter(&snapshot).flatten() {
                mask(&mut bytes, m.start(), m.end());
            }
            mask_re(
                &mut bytes,
                r"\]\((?:[^()\n]|\([^()\n]*\))*\)|\]\[[^]\n]*\]",
                1,
            );
        }
        mask_re(
            &mut bytes,
            r"(?:https?://|www\.)\S+|[\w.+-]+@[\w.-]+\.\w+",
            0,
        );
        Self {
            source,
            masked: String::from_utf8(bytes).unwrap(),
            ignored,
        }
    }
    pub fn ignored(&self, rule: &str, offset: usize) -> bool {
        self.ignored
            .iter()
            .any(|(r, a, b)| (r == rule || r == "Danish") && *a <= offset && offset < *b)
    }
    pub fn adjacent(&self, a: &Word, b: &Word) -> bool {
        static GAP: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"^[ \t]*(?:\*{1,2})?[ \t]*$").unwrap());
        GAP.is_match(&self.source[a.end..b.start])
    }
    pub fn location(&self, start: usize, end: usize) -> (usize, [usize; 2]) {
        let before = &self.source[..start];
        let line = before.bytes().filter(|b| *b == b'\n').count() + 1;
        let line_start = before.rfind('\n').map_or(0, |p| p + 1);
        (
            line,
            [
                self.source[line_start..start].chars().count() + 1,
                self.source[line_start..end].chars().count(),
            ],
        )
    }
    pub fn segments(&self) -> Vec<Vec<Word>> {
        static SEG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^\n.!?;]+").unwrap());
        static WORD: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"\p{L}+(?:[-'’]\p{L}+)*").unwrap());
        SEG.find_iter(&self.masked)
            .filter_map(|segment| {
                let words: Vec<Word> = WORD
                    .find_iter(segment.as_str())
                    .map(|m| Word {
                        text: m.as_str().to_owned(),
                        lower: m.as_str().to_lowercase(),
                        start: segment.start() + m.start(),
                        end: segment.start() + m.end(),
                    })
                    .collect();
                let da = words.iter().filter(|w| has(DA, &w.lower)).count();
                let en = words.iter().filter(|w| has(EN, &w.lower)).count();
                if words.is_empty() || (en >= 2 && en > da) {
                    None
                } else {
                    Some(words)
                }
            })
            .collect()
    }
}
