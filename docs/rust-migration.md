# Fra Python-prototype til Rust

Rust-versionen erstatter Python-CLI, spaCy, Vale-processen og den eksterne
Hunspell-proces. `src/` indeholder tekstbehandling, diagnostik, regler,
ordbogsadapter, installation og evaluering. Den tidligere implementation
findes i Git-historikken ved `a8bf898`.

Sætningsanalysen bruger UDPipes indbyggede C++-bibliotek via `udpipe-rs`.
Projektets egen kode er Rust; dette er ikke en omskrivning af alle
underliggende biblioteker til Rust. Den medfølgende DDT-model er trænet på
et fastlåst, åbent korpus og er CC BY-SA 4.0. Blogtekster og regeleksempler
indgår ikke i modellens træningsdata. Se [modelkortet](../models/README.md).

## Hvad er bevaret?

- Regel-id'er, de fire profiler og eksisterende `vale`-undtagelseskommentarer.
- Markdown-maskering, Unicode-positioner, tekst/JSON og exitkoderne 0/1/2.
- Ordlister og adskillelse mellem advarsler og ukendte ord.
- YAML/JSON-bidrag med kilde, forklaring, fejl og korrekte modeksempler.
- Lottie-kilden og README-animationen.

`--engine`, Python-installationsscripts og fuld Vale-kompatibilitet er fjernet.
Den native YAML-motor understøtter `existence` og `substitution` og afviser
ukendte felter. `--rules-dir` vælger egne regelpar, og `danish rule test`
kontrollerer deres eksempler. Se [brugsvejledningen](usage.md).

## Stavning og hastighed

Spellbook kontrollerer gyldige ord og produktive sammensætninger.
Rettelsesforslag findes først blandt nære COR-former. En fuld
sammensætningssøgning bruges kun som ekstra evidens for enkelte kandidater,
og gentagne opslag caches med en størrelsesgrænse. Tvetydige ukendte ord
bliver fortsat ikke automatisk til staveadvarsler.

To interne bogstavers forskel kræver yderligere støtte: bøjningsklasse,
gentagen bogstavgruppe eller ordbogens forslag. Et nærstående ord kan
stadig have en forkert betydning. Ukendte ords forslag kan være anderledes
end i Python-versionen og er ikke et facit.

## Evaluering

Målt 2026-10-26. Alle 14 Rust-testsuiter består, herunder CLI, kodeblokke,
Unicode-positioner, undtagelser, profiler og eksterne regelpakker.

| Kontrol | Python `a8bf898` | Rust `0.2.0-alpha.1` |
| --- | --- | --- |
| Oprindelige regeleksempler | 84/84 | 84/84 |
| Oprindelige kvalitetseksempler | 60/61 | 61/61 |
| Blogindlæg i kørslen | 179 | 180 |
| Standardadvarsler | 72 | 83 |
| Staveadvarsler | 24 | De samme 24 |
| Udviklerens vurdering: nyttige | 50 | 56 |
| Delvise rettelser | 2 | 6 |
| Kræver videre sprogfaglig vurdering | 18 | 20 |
| Forkerte forslag i gennemgangen | 2 | 1 |

De oprindelige 179 filer er uændrede, kontrolleret med SHA-256. Det ekstra
indlæg gav ingen advarsler. 69 gamle fund er bevaret, 14 er nye, og 3 er
fjernet. Af de 50 tidligere vurderede nyttige fund er 49 bevaret.

De tre fjernede fund er forskellige:

- `rent intuition → ren intuition`: et tidligere vurderet nyttigt fund,
  som nu udelades af den forsigtige biordsafgrænsning.
- `det indsats → den indsats`: tidligere til videre vurdering; meningen
  kunne også være *givet det en indsats*.
- `sjovt processen → sjove processen`: et forkert forslag, som er fjernet.

Nye fund omfatter `Konceptet var simpel → simpelt`, `mobile app → mobil app`,
`den bedst værdi → den bedste værdi` og `en menneske → et menneske`.
`et ærlig svar → et ærligt svar` findes nu i kvalitetseksemplerne.

Der er stadig et kendt forkert blogforslag: i `et ask` betyder *ask* en
engelsk anmodning, mens reglen bruger det danske trænavns køn. Flere andre
låneord kræver sprogfaglig vurdering. Reglerne foretager ingen automatisk rettelse.

[Alle 83 fund med vurderinger](rust-blog-findings.md) og
[JSON med diagnostik, eksempler, modeloplysninger og baseline-hashes](rust-evaluation.json)
gør gennemgangen efterprøvbar. Klassifikationerne er foretaget af samme
agent, som skrev reglerne; de er ikke en måling af præcision eller dækning.

Den samlede native kørsel tog cirka 90,5 sekunder inklusive kvalitetseksempler
og ukendte ord. Stavning alene tog cirka 3,7 sekunder på de 180 filer.
Det er lokale observationer under udvikling, ikke et kontrolleret
hastighedsbenchmark. De 1.083 ukendte ord er særskilte `suggestion`-fund,
ikke erklærede fejl. Inputfilerne er kontrolleret uændrede efter kørslen.

Reproducer på dit eget udvalg efter `danish setup`:

```sh
danish evaluate --blog-root /sti/til/blog-posts --output rapport.json
```

CLI'en er også kørt med `PATH=/nonexistent`, friskinstallerede ordbøger og
den indbyggede model. Både stave- og grammatikforslag fungerede uden eksterne programmer.

Testmaterialet er udviklingsmateriale, ikke et uafhængigt benchmark.
En vellykket portering dokumenterer ikke fuld dansk korrektur. Projektet
har fortsat brug for sprogfaglige reviewers og et separat, annoteret korpus.
