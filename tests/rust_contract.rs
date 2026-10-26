use pks_agent_danish::{
    checker::{Checker, Options},
    prose::Prose,
    resources,
    rules::Rules,
};
use serde_json::Value;
use std::{collections::HashSet, path::PathBuf};
fn checker() -> Checker {
    let data = resources::directory();
    Checker::load(&data, None, None).expect("Run danish setup first")
}
#[test]
fn bundled_rule_examples() {
    let rules = Rules::load(None).unwrap();
    let mut count = 0;
    for rule in rules.rules {
        assert!(!rule.record.autofix);
        assert!(!rule.record.source.contains("TODO"));
        for case in &rule.record.cases {
            let doc = Prose::new(&case.text, true);
            let actual = rule.check(&doc, &HashSet::new());
            assert_eq!(
                actual.iter().map(|a| &a.r#match).collect::<Vec<_>>(),
                case.matches.iter().collect::<Vec<_>>(),
                "{}: {}",
                rule.record.id,
                case.text
            );
            for a in actual {
                assert_eq!(
                    case.text
                        .chars()
                        .skip(a.start)
                        .take(a.end - a.start)
                        .collect::<String>(),
                    a.r#match
                );
                assert_eq!(
                    case.text
                        .lines()
                        .nth(a.line - 1)
                        .unwrap()
                        .chars()
                        .skip(a.span[0] - 1)
                        .take(a.span[1] - a.span[0] + 1)
                        .collect::<String>(),
                    a.r#match
                );
            }
            count += 1;
        }
    }
    assert!(count >= 84, "The original examples must be retained");
}
#[test]
fn quality_corpus() {
    let c = checker();
    let fixture: Value = serde_json::from_str(include_str!("quality.json")).unwrap();
    let mut failures = vec![];
    for case in fixture["cases"].as_array().unwrap() {
        let text = case["text"].as_str().unwrap();
        let actual = c.check(text, &Options::default()).unwrap();
        let expected = case["findings"].as_array().unwrap();
        let correct = expected.len() == actual.len()
            && expected.iter().all(|e| {
                actual.iter().any(|a| {
                    Some(a.check.as_str()) == e[0].as_str()
                        && Some(a.r#match.as_str()) == e[1].as_str()
                        && (e[2].is_null()
                            || a.suggestions
                                .iter()
                                .any(|s| Some(s.as_str()) == e[2].as_str()))
                })
            });
        if !correct {
            failures.push(format!(
                "{text}\n  Expected: {expected:?}\n  Actual: {actual:?}"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn source_positions_code_unicode_and_suppression() {
    let c = checker();
    let text = "---\ntitle: \"Hun har interresse.\"\nslug: interresse\n---\n\n```sessionstory\nHan plejedede at komme.\n```\n\n😀 Åse siger: Han plejedede at komme.\n<!-- vale Danish.Spelling = NO -->\nHan plejedede at komme.\n<!-- vale Danish.Spelling = YES -->\nVi læser `plejedede` på https://example.com/plejedede\n";
    let actual = c.check(text, &Options::default()).unwrap();
    assert_eq!(actual.len(), 2, "{actual:?}");
    assert_eq!(actual[0].line, 2);
    assert_eq!(actual[1].line, 10);
    for a in actual {
        assert_eq!(
            text.chars()
                .skip(a.start)
                .take(a.end - a.start)
                .collect::<String>(),
            a.r#match
        );
        assert_eq!(
            text.lines()
                .nth(a.line - 1)
                .unwrap()
                .chars()
                .skip(a.span[0] - 1)
                .take(a.span[1] - a.span[0] + 1)
                .collect::<String>(),
            a.r#match
        );
    }
    let suppressed = "<!-- vale Dansk.Interesse = NO -->\nHun har interresse.\n<!-- vale Dansk.Interesse = YES -->";
    assert!(c.check(suppressed, &Options::default()).unwrap().is_empty());
    let plain = c
        .check(
            "Hun har `ABONNOMENT`.",
            &Options {
                markdown: false,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(plain.len(), 1);
    assert_eq!(plain[0].check, "Dansk.Abonnement");
}
#[test]
fn false_positive_regressions() {
    let c = checker();
    for text in [
        "Det er den rigtige konklusion.",
        "Det er det egentlige produkt.",
        "Den bedst mulige løsning virker.",
        "Den billigst mulige bro virker.",
        "Den \"rigtige linje\" er her.",
        "Er det god kørsel?",
        "Ellers er den ren dødvægt.",
        "Vi bruger en typet kontrakt.",
        "Traefik kører på serveren.",
        "Vi læser stdout fra processen.",
        "Hetzner holder serveren kørende.",
        "Den triggerer en hændelse.",
    ] {
        assert!(
            c.check(text, &Options::default()).unwrap().is_empty(),
            "{text}"
        );
    }
}
#[test]
fn profiles_and_check_filters() {
    let c = checker();
    let text = "Vi fremsender hermed vores features med interresse.";
    for (profile, expected) in [("dansk", 1), ("klart-sprog", 2), ("agentics", 3)] {
        let r = c
            .check(
                text,
                &Options {
                    profile: profile.into(),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(r.len(), expected, "{r:?}");
    }
    let text = "Hun har interresse. Modellerne er dygtig.";
    let r = c
        .check(
            text,
            &Options {
                checks: "grammar".into(),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].check, "Danish.PredicateAgreement");
    let r = c
        .check(
            text,
            &Options {
                checks: "spelling".into(),
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].check, "Dansk.Interesse");
}
#[test]
fn unknowns_and_dictionary() {
    let c = checker();
    let s = "Vi bruger xyzqwerty til det.";
    assert!(c.check(s, &Options::default()).unwrap().is_empty());
    let r = c
        .check(
            s,
            &Options {
                uncertain: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].severity, "suggestion");
    let allowed = HashSet::from(["xyzqwerty".into(), "plejedede".into()]);
    let r = c
        .check(
            "Han plejedede at bruge xyzqwerty til det.",
            &Options {
                allowed,
                uncertain: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert!(r.is_empty(), "{r:?}");
}
#[test]
fn experimental_rule_workflow() {
    let dir = tempfile::tempdir_in(std::env::current_dir().unwrap().join("target")).unwrap();
    let id = "Experimental.TestRule";
    pks_agent_danish::rules::new_rule(id, dir.path()).unwrap();
    assert!(pks_agent_danish::rules::new_rule(id, dir.path()).is_err());
    assert!(pks_agent_danish::rules::new_rule("../Escape", dir.path()).is_err());
    let r = Rules::load(Some(dir.path())).unwrap();
    let result = r.check(
        &Prose::new("Her er EKSEMPELMOENSTER.", true),
        "experimental",
        "all",
        &HashSet::new(),
    );
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].check, id);
    assert!(r.test_examples().unwrap() >= 87);
    let path = dir.path().join("rules/Experimental.TestRule.json");
    let mut metadata: Value =
        serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    metadata["cases"][0]["matches"] = serde_json::json!([]);
    std::fs::write(path, serde_json::to_string(&metadata).unwrap()).unwrap();
    assert!(
        Rules::load(Some(dir.path()))
            .unwrap()
            .test_examples()
            .is_err()
    );
}
#[test]
fn cli_contract_and_no_writes() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let binary = env!("CARGO_BIN_EXE_danish");
    let run = |args: &[&str], input: &str| {
        let mut child = Command::new(binary)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    };
    let r = run(
        &["lint", "-", "--format", "json", "--fail-on", "warning"],
        "Modellerne er dygtig.",
    );
    assert_eq!(
        r.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&r.stderr)
    );
    let json: Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(json["stdin.md"][0]["Check"], "Danish.PredicateAgreement");
    assert_eq!(
        run(&["lint", "-"], "Alt er i orden.").status.code(),
        Some(0)
    );
    assert_eq!(run(&["lint", "missing.md"], "").status.code(), Some(2));
    assert_eq!(run(&["lint", "-", "file.md"], "").status.code(), Some(2));
    assert_eq!(run(&["explain", "Unknown.Rule"], "").status.code(), Some(2));
    let dir = tempfile::tempdir_in(std::env::current_dir().unwrap().join("target")).unwrap();
    let path = dir.path().join("tekst med mellemrum.md");
    let content = "Åse har interresse.";
    std::fs::write(&path, content).unwrap();
    let r = run(&["lint", path.to_str().unwrap()], "");
    assert_eq!(r.status.code(), Some(1));
    assert_eq!(std::fs::read_to_string(path).unwrap(), content);
}
#[test]
fn resource_corruption_is_detected() {
    let dir = tempfile::tempdir_in(std::env::current_dir().unwrap().join("target")).unwrap();
    let r = resources::records().remove(0);
    let p = dir.path().join(&r.file);
    std::fs::write(&p, "bad dictionary").unwrap();
    assert!(!resources::verify(&p, &r));
}

#[test]
fn original_profile_corpus() {
    let rules = Rules::load(None).unwrap();
    let expected: Value = serde_json::from_str(include_str!("corpus/expected.json")).unwrap();
    for (profile, files) in expected.as_object().unwrap() {
        for (name, checks) in files.as_object().unwrap() {
            let source = std::fs::read_to_string(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/corpus")
                    .join(name),
            )
            .unwrap();
            let mut actual = rules
                .check(&Prose::new(&source, true), profile, "all", &HashSet::new())
                .into_iter()
                .map(|a| a.check)
                .collect::<Vec<_>>();
            actual.sort();
            let mut wanted = checks
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_owned())
                .collect::<Vec<_>>();
            wanted.sort();
            assert_eq!(actual, wanted, "{profile}/{name}");
        }
    }
}

#[test]
fn dictionary_adapter_preserves_literal_slashes() {
    assert_eq!(
        pks_agent_danish::language::normalize_numeric_dictionary("2\nA/S\nhus/1,2\n"),
        "2\nA\\/S\nhus/1,2"
    );
}

#[test]
fn custom_rule_ids_cannot_escape_the_directory() {
    let dir = tempfile::tempdir_in(std::env::current_dir().unwrap().join("target")).unwrap();
    pks_agent_danish::rules::new_rule("Experimental.Example", dir.path()).unwrap();
    let path = dir.path().join("rules/Experimental.Example.json");
    let mut record: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    record["id"] = Value::String("../Escape.Other".into());
    std::fs::write(path, serde_json::to_string(&record).unwrap()).unwrap();
    assert!(Rules::load(Some(dir.path())).is_err());
}

#[test]
fn spelling_candidates_do_not_invent_unrelated_two_letter_deletions() {
    let c = checker();
    let options = Options {
        checks: "spelling".into(),
        ..Default::default()
    };
    assert!(
        c.check("Vi skal perkulere over det.", &options)
            .unwrap()
            .is_empty()
    );
    let result = c.check("Han plejedede at komme.", &options).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].suggestions, ["plejede"]);
    for (text, expected) in [
        ("Det er tragedisk.", "tragisk"),
        ("Han indrømmetede det.", "indrømmede"),
    ] {
        let result = c.check(text, &options).unwrap();
        assert_eq!(result.len(), 1, "{text}");
        assert_eq!(result[0].suggestions, [expected]);
    }
}

#[test]
fn native_model_ambiguity_regressions() {
    let c = checker();
    for text in [
        "Vi træffer konstant beslutninger.",
        "Brugen steg jævnt uge for uge.",
        "Det erstatter sjældent disciplin.",
        "Vis billederne.",
        "Vi kan se det rå.",
        "Der kører en e2e-test.",
        "Vi bruger en v6-adresse.",
        "Det er et *idé*-problem.",
        "Den gemmer et rolling buffer.",
        "Det er perfekt designet.",
        "Der er 3 milliarder aktive pr. token.",
        "Vi starter nye vibe coding-projekter.",
        "Bare udførelse.",
        "Stadig intet.",
        "Det modellen kom med, var godt.",
        "Det alle tror, er forkert.",
        "Vi bruger en C#-funktion.",
        "Vi bruger en F#-fil.",
        "Vi vil gøre det sjovt processen igennem.",
    ] {
        let result = c.check(text, &Options::default()).unwrap();
        assert!(result.is_empty(), "{text}: {result:?}");
    }
    for (text, id, matched, suggestion) in [
        (
            "Det kontekst betyder noget.",
            "Danish.ArticleGender",
            "Det",
            "Den",
        ),
        (
            "Det smiley tager kanten af.",
            "Danish.ArticleGender",
            "Det",
            "Den",
        ),
        (
            "Den enarmet tyveknægt.",
            "Danish.AdjectiveAgreement",
            "enarmet",
            "enarmede",
        ),
        (
            "Klart kontekst.",
            "Danish.AdjectiveAgreement",
            "Klart",
            "Klar",
        ),
        ("Godt kontekst.", "Danish.AdjectiveAgreement", "Godt", "God"),
        (
            "Enormt gevinst.",
            "Danish.AdjectiveAgreement",
            "Enormt",
            "Enorm",
        ),
    ] {
        let result = c.check(text, &Options::default()).unwrap();
        assert_eq!(result.len(), 1, "{text}: {result:?}");
        assert_eq!(result[0].check, id);
        assert_eq!(result[0].r#match, matched);
        assert!(result[0].suggestions.contains(&suggestion.to_owned()));
    }
}
