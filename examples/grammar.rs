use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let path = args.next().context(
        "Brug: cargo run --release --example grammar -- models/danish-ddt.udpipe 'Din sætning.'",
    )?;
    let sentences: Vec<_> = args.collect();
    anyhow::ensure!(
        !sentences.is_empty(),
        "Angiv mindst én sætning efter modellen"
    );
    let model = udpipe_rs::Model::load(path)?;
    for sentence in sentences {
        println!("{sentence}");
        for token in model.parse(&sentence)? {
            println!(
                "{} {} {} -> {} {}",
                token.id, token.form, token.upostag, token.head, token.deprel
            );
        }
    }
    Ok(())
}
