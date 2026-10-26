# pks-agent-danish

![Dansk, med omtanke. Tre danske sætninger får markeret bøjningsfejl og vist rettelsesforslag.](docs/assets/danish-lint.gif)

[![Tests](https://github.com/pksorensen/pks-agent-danish/actions/workflows/test.yml/badge.svg)](https://github.com/pksorensen/pks-agent-danish/actions/workflows/test.yml)
[![Status: alpha](https://img.shields.io/badge/status-alpha-8b5cf6)](#hvor-god-er-den)
[![License: MIT](https://img.shields.io/badge/code-MIT-168367)](LICENSE)

**En lokal linter til dansk tekst. Forklarlige forslag — og plads til din stemme.**

Finder udvalgte stave- og grammatikfejl i Markdown og ren tekst. Kører på
din computer, sender ikke teksten til en tjeneste og ændrer ingen filer.
Målet er et åbent dansk sprogværktøj, som bliver bedre med fælles eksempler,
modeksempler og sprogfaglig viden.

**Alpha:** Den finder reelle fejl, men kan også overse fejl og foreslå noget forkert.
Prøv den på en tekst, du kender, og [fortæl os, om den var nyttig](https://github.com/pksorensen/pks-agent-danish/issues/new?template=feedback.yml).

[Lottie-kilde](docs/assets/danish-lint.json) · [Statisk billede](docs/assets/danish-lint-poster.png) · [Om animationen](docs/animation.md)

## Prøv den

Kræver **Python 3.12** og det native Hunspell-bibliotek. Linux er afprøvet;
macOS-installation er beskrevet, men endnu ikke verificeret. På Windows kan WSL bruges.

```sh
git clone https://github.com/pksorensen/pks-agent-danish.git
cd pks-agent-danish

# Ubuntu/Debian:
sudo apt-get install libhunspell-1.7-0
# macOS: brew install hunspell

python3.12 scripts/setup.py
bin/danish lint din-tekst.md
```

Setup opretter et isoleret miljø og henter fastlåste sprogdata, model og
værktøjer. Første installation kræver netværk; selve tekstkontrollen gør ikke.
Data ligger under `data/pks-agent-danish/` i et selvstændigt checkout.
Systemets Python og PATH ændres ikke.

Prøv uden at oprette en fil:

```sh
bin/danish lint - <<'TEXT'
Modellerne er dygtig. Det er et workshop.
TEXT
```

```text
warning[Danish.PredicateAgreement]: Tillægsordet skal passe til 'Modellerne'; overvej 'dygtige'.
warning[Danish.ArticleGender]: 'workshop' har fk-køn i COR; overvej 'en'.
```

Outputtet ovenfor er forkortet; CLI'en viser også fil, linje og kolonne.
Animationen illustrerer forslagene. Produktet er en CLI og har ikke en grafisk editor.

## Hvad finder den?

| Eksempel | Forslag |
| --- | --- |
| Modellerne er **dygtig**. | dygtige |
| Det er **et** workshop. | en |
| Et **rigtig** produkt. | rigtigt |
| Hun **reparere** cyklen. | reparerer |
| Han **plejedede** at komme. | plejede |

Motoren kombinerer Stavekontrolden/Hunspell, Dansk Sprognævns COR,
spaCys danske sætningsanalyse og Vale. Reglerne undersøger afgrænsede
konstruktioner. Der er endnu ingen fuld kommakontrol eller forståelse af
hele tekstens betydning. Stil- og terminologipakker er tilvalg.

```sh
bin/danish lint --format json artikel.md
bin/danish lint --checks grammar artikel.md
bin/danish lint --profile klart-sprog artikel.md
bin/danish lint --dictionary vocabularies/development.txt artikel.md
bin/danish explain Danish.PredicateAgreement
```

[CLI, profiler og undtagelser](docs/usage.md) · [Grammatikregler](docs/grammar.md) · [Vale-regelkatalog](docs/rules.md)

## Hvor god er den?

På **179 blogindlæg** gav prototypen **72 advarsler**. Udviklerens gennemgang:
50 nyttige lokale rettelser, 2 delrettelser, 18 uafklarede fund og 2 forkerte
rettelsesforslag. Ingen særlig ordliste blev brugt til denne kørsel.

Det er **ikke et uafhængigt benchmark**: Bloggen blev brugt under udviklingen,
og vurderingen er lavet af samme AI-agent, som implementerede reglerne.
Dækningen af samtlige fejl er ikke målt. Tekniske låneord er en kendt fejlkilde.

De medfølgende 61 kvalitetseksempler har 60 forventede udfald. Den kendte
mangel — `Hun gav et ærlig svar` — er bevaret som en forventet fejl i testene.

[Hele evalueringen](docs/evaluation.md) · [Alle 72 fund og vurderinger](docs/blog-findings.md)

## Byg bedre dansk sammen med os

Du behøver ikke kunne kode for at hjælpe:

- **[En forkert advarsel](https://github.com/pksorensen/pks-agent-danish/issues/new?template=false-positive.yml)** — vis en sætning, vi skal lade være.
- **[En fejl vi overser](https://github.com/pksorensen/pks-agent-danish/issues/new?template=rule.yml)** — vis fejlen og den ønskede rettelse.
- **[Erfaringer fra din tekst](https://github.com/pksorensen/pks-agent-danish/issues/new?template=feedback.yml)** — hvad hjalp, hvad støjede, og ville du bruge den igen?

Issues er offentlige, så brug korte, anonymiserede eksempler. Vi søger især
sprogfaglige reviewers og folk, der skriver andre typer tekster end tekniske blogs.

Simple mønstre kan bidrages i YAML; grammatikregler bruger Python og sproglige
analyser. Hver regel skal have en kilde, fejl, korrekte modeksempler og kendte
undtagelser. Se [bidragsvejledningen](CONTRIBUTING.md).

```sh
bin/danish rule new Experimental.MinRegel
python3 scripts/test.py
python3 scripts/catalog.py --check
```

## Licens og inspiration

Inspireret af [english-rs](https://github.com/Aaryan-Kapoor/english-rs).
Kode, selvstændige regler og animation er [MIT-licenseret](LICENSE).
Ordbøger, analysemodel og skrifttype har egne licenser — se [THIRD_PARTY.md](THIRD_PARTY.md).
