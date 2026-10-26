# Brug af Rust-CLI og regelpakker

Installér med Cargo eller en native udgivelse; kør `danish setup` én gang.
`danish lint` bruger kun lokale filer og opretter ingen netværksforbindelse.
Setup downloader ikke programkode, og lint ændrer ikke teksten.

## Profiler og regler

| Profil | Simple regler, ud over grammatik og ordbogskontrol |
| --- | --- |
| `dansk` (standard) | Fire præcise stavemønstre |
| `klart-sprog` | Desuden fire forslag til enklere formuleringer |
| `agentics` | Desuden to organisationsspecifikke termvalg |
| `experimental` | Experimental-regler fra `--rules-dir`; grammatikkontrollen er stadig aktiv |

`--checks grammar` kører kun grammatik; `--checks spelling` kører kun stavning.
`--include-uncertain` viser ukendte ord som `suggestion`. Ukendt betyder ikke forkert.
Den engelske hjælpeordbog dæmper støj fra låneord, men kan også skjule danske fejl,
som tilfældigvis er engelske ord.

```sh
danish lint --profile agentics --format json artikel.md anden-tekst.txt
danish lint --checks grammar artikel.md
danish lint --include-uncertain artikel.md
danish lint --dictionary mine-ord.txt artikel.md
```

En lokal ordliste indeholder ét ord pr. linje; `#` indleder kommentarer.
Ordlisten undertrykker fund på det angivne ord.

## Input og kildepositioner

Angiv `.md`- og `.txt`-filer eksplicit eller via shell-glob. Standard input
(`-`) læses som Markdown og skal bruges alene. Kode, URL'er, linkdestinationer
og strukturelle metadata maskeres. Titel og beskrivelse kontrolleres.
En simpel sproglig heuristik springer engelsktunge tekststykker over.
Det er ikke fuld sproggenkendelse eller understøttelse af alle MDX/JSX-varianter.

JSON indeholder `Check`, `Severity`, `Message`, `Match`, `Suggestions`, `Line`
og `Span` i originalteksten. Linjer/kolonner er 1-baserede, slutkolonnen er
inkluderet. `Start`/`End` tæller Unicode-skalaværdier med eksklusiv slutposition,
som i den tidligere Python-CLI. Det er **ikke bytepositioner eller JavaScript
UTF-16-positioner**. Rust bruger UTF-8-bytepositioner internt og konverterer
ved rapportering. Analysemodellen må ikke placere et fund på ændret tekst.

## Undtagelser og exitkoder

```md
<!-- vale Danish.ArticleGender = NO -->
Her står et tilsigtet citat med et workshop.
<!-- vale Danish.ArticleGender = YES -->
```

Syntaksen er bevaret for eksisterende tekster; `danish` kan også bruges i
stedet for `vale`. Brug `Danish` til at undtage alle fund i en passage.
Præcise id'er som `Dansk.Interesse` understøttes fortsat. Der er ingen `--fix`.

Exitkoder: `0` = ingen fund på valgt fejlniveau, `1` = fund på/over niveauet,
`2` = input- eller værktøjsproblem. Standard `--fail-on error` viser advarsler
uden at fejle CI; vælg `--fail-on warning` for at gøre det.

## Data, modeller og brugerregler

`danish setup` lægger data under operativsystemets lokale applikationsdata,
fx `~/.local/share/pks-agent-danish/resources` på Linux. `--resources` eller
`PKS_DANISH_RESOURCES` erstatter denne mappe. De skal pege på selve mappen
med `.aff`, `.dic` og `.tsv`, ikke dens overmappe. `--model` kan vælge en
anden lokal UDPipe-model til udvikling; standardmodellen er indbygget.

```sh
danish rule new Experimental.MinRegel --root mine-regler
# Udfyld YAML, JSON, kilder og eksempler:
danish --rules-dir mine-regler rule test
danish lint --profile experimental --rules-dir mine-regler artikel.md
```

Medfølgende regler og beskrivelser er indbygget i binæren, så den virker fra
vilkårlige mapper. `--rules-dir` læser `rules/*.json` og
`styles/<pakke>/<navn>.yml`. Et eksplicit valgt bibliotek kan også erstatte
en indbygget regel med samme id. Den native motor understøtter `existence` og
`substitution` med felterne `message`, `level`, `ignorecase`, `tokens` og `swap`.
Ukendte felter/regeltyper giver en fejl. Det er en afgrænset kompatibel del af
Vale-formatet, ikke en genimplementering af hele Vale. Regex følger Rusts
regex-syntaks (ingen backreferences eller lookaround i brugerregler).

## Ændringer fra Python-alphaen

`--engine hybrid|vale`, `PKS_DANISH_VALE` og `PKS_DANISH_HUNSPELL` er fjernet.
Motoren er altid native. Brug `danish setup`, `cargo test`, `danish catalog`
og `danish evaluate` i stedet for Python-scripts. Regel-id'er, profiler,
JSON-spans og eksisterende undtagelseskommentarer er bevaret. Den nye model
og stavemotor kan give andre fund; se [porteringsevalueringen](rust-migration.md).
