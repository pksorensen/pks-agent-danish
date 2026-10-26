use anyhow::{Context, Result, bail};
use clap::{Args, Parser, Subcommand};
use pks_agent_danish::{
    checker::{CORE, Checker, Options},
    diagnostic::Diagnostic,
    resources,
    rules::{self, Rules},
};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::Read,
    path::PathBuf,
};
#[derive(Parser)]
#[command(version, about = "Lokale, forklarlige regler for dansk tekst")]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[arg(long, global = true)]
    resources: Option<PathBuf>,
    #[arg(long, global = true)]
    model: Option<PathBuf>,
    #[arg(long, global = true)]
    rules_dir: Option<PathBuf>,
}
#[derive(Subcommand)]
enum Command {
    Setup,
    Lint(Lint),
    Rules,
    Explain {
        id: String,
    },
    Rule {
        #[command(subcommand)]
        action: RuleAction,
    },
    Evaluate {
        #[arg(long)]
        blog_root: Option<PathBuf>,
        #[arg(long)]
        output: PathBuf,
    },
    Catalog {
        #[arg(long)]
        check: bool,
        #[arg(long, default_value = "docs/rules.md")]
        output: PathBuf,
    },
}
#[derive(Subcommand)]
enum RuleAction {
    Test,
    New {
        id: String,
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
}
#[derive(Args)]
struct Lint {
    #[arg(required = true)]
    paths: Vec<String>,
    #[arg(long,default_value="dansk",value_parser=["dansk","klart-sprog","agentics","experimental"])]
    profile: String,
    #[arg(long,default_value="text",value_parser=["text","json"])]
    format: String,
    #[arg(long,default_value="error",value_parser=["suggestion","warning","error"])]
    fail_on: String,
    #[arg(long,default_value="all",value_parser=["all","grammar","spelling"])]
    checks: String,
    #[arg(long)]
    include_uncertain: bool,
    #[arg(long)]
    dictionary: Option<PathBuf>,
}
fn main() {
    if let Err(e) = run() {
        eprintln!("Fejl: {e:#}");
        std::process::exit(2);
    }
}
fn run() -> Result<()> {
    let cli = Cli::parse();
    let data = cli.resources.unwrap_or_else(resources::directory);
    match cli.command {
        Command::Setup => resources::setup(&data)?,
        Command::Rules => {
            for (id, text) in CORE {
                println!("{id} (experimental): {text}");
            }
            for r in Rules::load(cli.rules_dir.as_deref())?.rules {
                println!(
                    "{} ({}): {}",
                    r.record.id, r.record.status, r.record.summary
                );
            }
        }
        Command::Explain { id } => {
            if let Some((_, text)) = CORE.iter().find(|(r, _)| *r == id) {
                println!(
                    "{id}: {text}\n\nKilde: COR, Stavekontrolden og DDT. Eksperimentel, afgrænset konstruktion.\nIngen automatisk rettelse. Se docs/grammar.md."
                );
            } else {
                let rules = Rules::load(cli.rules_dir.as_deref())?;
                let r = &rules
                    .rules
                    .iter()
                    .find(|r| r.record.id == id)
                    .context("Ukendt regel")?
                    .record;
                println!(
                    "{}: {}\n\n{}\n\nKilde: {}\n{}\n\nUndtagelser: {}",
                    r.id,
                    r.summary,
                    r.explanation,
                    r.source,
                    r.source_note,
                    r.exceptions.join(" ")
                );
            }
        }
        Command::Rule {
            action: RuleAction::New { id, root },
        } => rules::new_rule(&id, &root)?,
        Command::Rule {
            action: RuleAction::Test,
        } => {
            let count = Rules::load(cli.rules_dir.as_deref())?.test_examples()?;
            println!("{count} regeleksempler bestod.");
        }
        Command::Catalog { check, output } => {
            let content = catalog(&Rules::load(cli.rules_dir.as_deref())?);
            if check {
                if fs::read_to_string(&output)? != content {
                    bail!("Regeloversigten er forældet. Kør danish catalog");
                }
                println!("Regeloversigten er ajour.");
            } else {
                fs::write(output, content)?;
            }
        }
        Command::Lint(args) => {
            if args.paths.iter().any(|p| p == "-") && args.paths.len() != 1 {
                bail!("Standard input (-) skal bruges alene");
            }
            let mut sources = vec![];
            for name in &args.paths {
                if name == "-" {
                    let mut s = String::new();
                    std::io::stdin().read_to_string(&mut s)?;
                    sources.push(("stdin.md".to_owned(), s, true));
                } else {
                    let path = PathBuf::from(name);
                    let ext = path
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if !path.is_file() || !matches!(ext.as_str(), "md" | "txt") {
                        bail!("Forventede en eksisterende .md- eller .txt-fil: {name}");
                    }
                    sources.push((name.clone(), fs::read_to_string(path)?, ext == "md"));
                }
            }
            let allowed = if let Some(path) = args.dictionary {
                fs::read_to_string(path)?
                    .lines()
                    .map(str::trim)
                    .filter(|s| !s.is_empty() && !s.starts_with('#'))
                    .map(str::to_lowercase)
                    .collect()
            } else {
                HashSet::new()
            };
            let checker = Checker::load(&data, cli.model.as_deref(), cli.rules_dir.as_deref())?;
            let mut o = Options {
                checks: args.checks,
                profile: args.profile,
                uncertain: args.include_uncertain,
                allowed,
                ..Default::default()
            };
            let mut report: BTreeMap<String, Vec<Diagnostic>> = BTreeMap::new();
            for (name, source, markdown) in sources {
                o.markdown = markdown;
                report.insert(
                    name.clone(),
                    checker.check(&source, &o).with_context(|| name)?,
                );
            }
            if args.format == "json" {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                for (name, alerts) in &report {
                    for a in alerts {
                        println!(
                            "{name}:{}:{}: {}[{}]: {}",
                            a.line, a.span[0], a.severity, a.check, a.message
                        );
                        if !a.suggestions.is_empty() {
                            println!("  forslag: {}", a.suggestions.join(", "));
                        }
                    }
                }
                if report.values().all(Vec::is_empty) {
                    println!("Ingen fund med den valgte regelpakke.");
                }
            }
            fn level(s: &str) -> u8 {
                match s {
                    "error" => 2,
                    "warning" => 1,
                    _ => 0,
                }
            }
            if report
                .values()
                .flatten()
                .any(|a| level(&a.severity) >= level(&args.fail_on))
            {
                std::process::exit(1);
            }
        }
        Command::Evaluate { blog_root, output } => {
            evaluate(&data, cli.model, cli.rules_dir, blog_root, output)?
        }
    }
    Ok(())
}
fn catalog(rules: &Rules) -> String {
    let mut s = String::from(
        "# Regeloversigt\n\nGenereret fra `rules/*.json` med `danish catalog`.\nRedigér regelbeskrivelserne, og generér denne fil igen.\n\nStartreglerne er afgrænsede og har endnu ikke eksternt sprogfagligt review.\n\n",
    );
    for item in &rules.rules {
        let r = &item.record;
        let link = if r.source.starts_with("https://") {
            r.source.clone()
        } else {
            format!("../{}", r.source)
        };
        s += &format!(
            "## {}\n\n{}\n\nKategori: `{}`. Status: `{}`. Automatisk rettelse: nej.\n\n{}\n\n[Kilde]({}). {}\n\nUndtagelser: {}\n\nSe [testeksemplerne](../rules/{}.json).\n\n",
            r.id,
            r.summary,
            r.category,
            r.status,
            r.explanation,
            link,
            r.source_note,
            r.exceptions.join(" "),
            r.id
        );
    }
    s.trim_end().to_owned() + "\n"
}
fn evaluate(
    data: &std::path::Path,
    model: Option<PathBuf>,
    rules: Option<PathBuf>,
    blog_root: Option<PathBuf>,
    output: PathBuf,
) -> Result<()> {
    use serde_json::{Value, json};
    use sha2::{Digest, Sha256};
    let start = std::time::Instant::now();
    let checker = Checker::load(data, model.as_deref(), rules.as_deref())?;
    let cases: Value = serde_json::from_str(include_str!("../tests/quality.json"))?;
    let mut quality = vec![];
    for case in cases["cases"].as_array().unwrap() {
        let actual = checker.check(case["text"].as_str().unwrap(), &Options::default())?;
        let expected = case["findings"].as_array().unwrap();
        let passed = actual.len() == expected.len()
            && expected.iter().all(|e| {
                actual.iter().any(|a| {
                    a.check == e[0].as_str().unwrap()
                        && a.r#match == e[1].as_str().unwrap()
                        && (e[2].is_null()
                            || a.suggestions
                                .iter()
                                .any(|s| Some(s.as_str()) == e[2].as_str()))
                })
            });
        quality.push(json!({"case":case,"actual":actual,"passed":passed}));
    }
    let mut files = vec![];
    let mut findings = vec![];
    let mut paths = vec![];
    if let Some(root) = blog_root {
        for entry in fs::read_dir(root)? {
            let path = entry?.path().join("da.md");
            if path.is_file() {
                paths.push(path);
            }
        }
        if paths.is_empty() {
            bail!("Ingen */da.md fundet");
        }
    }
    paths.sort();
    for (index, path) in paths.iter().enumerate() {
        let bytes = fs::read(path)?;
        let digest = format!("{:x}", Sha256::digest(&bytes));
        let source = std::str::from_utf8(&bytes)?;
        let alerts = checker
            .check(
                source,
                &Options {
                    uncertain: true,
                    ..Default::default()
                },
            )
            .with_context(|| format!("Kontrol af {}", path.display()))?;
        for a in alerts {
            let matched: String = source.chars().skip(a.start).take(a.end - a.start).collect();
            if matched != a.r#match {
                bail!("Forkert kildeposition");
            }
            let mut value = serde_json::to_value(&a)?;
            value["file"] = json!(path);
            value["context"] = json!(
                source
                    .chars()
                    .skip(a.start.saturating_sub(60))
                    .take(a.end - a.start + 140)
                    .collect::<String>()
            );
            findings.push(value);
        }
        if fs::read(path)? != bytes {
            bail!("Inputfilen ændrede sig under evalueringen");
        }
        files.push(json!({"file":path,"sha256":digest}));
        if (index + 1) % 40 == 0 {
            eprintln!("Kontrolleret {}/{}", index + 1, paths.len());
        }
    }
    // Verify all files again at the end, including ones checked early in the run.
    for file in &files {
        let path = file["file"].as_str().context("Korpusfilens sti")?;
        if format!("{:x}", Sha256::digest(fs::read(path)?)) != file["sha256"].as_str().unwrap() {
            bail!("Korpusfilen ændrede sig under evalueringen: {path}");
        }
    }
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for a in &findings {
        *counts
            .entry(a["Check"].as_str().unwrap().to_owned())
            .or_default() += 1;
    }
    let summary = json!({"version":env!("CARGO_PKG_VERSION"),"engine":"rust-udpipe-spellbook","files":files.len(),"warnings":findings.iter().filter(|a|a["Severity"]!="suggestion").count(),"by_rule":counts,"quality_passed":quality.iter().filter(|q|q["passed"]==true).count(),"quality_cases":quality.len(),"seconds":start.elapsed().as_secs_f64(),"source_unchanged":true});
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::write(
        output,
        serde_json::to_string_pretty(
            &json!({"summary":summary,"quality":quality,"files":files,"findings":findings}),
        )? + "\n",
    )?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}
