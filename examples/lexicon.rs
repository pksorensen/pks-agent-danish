fn main() -> anyhow::Result<()> {
    let language =
        pks_agent_danish::language::Language::load(&pks_agent_danish::resources::directory())?;
    let mut words: Vec<_> = std::env::args().skip(1).collect();
    if words.is_empty() {
        words = ["plejedede", "utrodeligt", "interresse", "konsolen"]
            .map(str::to_owned)
            .to_vec();
    }
    for word in words {
        println!(
            "{word} {} {:?}",
            language.spelled(&word),
            language.suggestions(&word)
        );
    }
    Ok(())
}
