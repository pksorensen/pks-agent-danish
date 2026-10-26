# Brug af CLI og regelpakker

Kommandoerne køres fra repositoryets rod efter installation.

## Hvad kontrolleres?

| Kontrol | Metode og afgrænsning |
| --- | --- |
| Stavning | Kendte fejlstavninger og konservative ordbogsforslag. Langt fra alle ukendte ord er fejl. |
| Nutids-r | Nogle konstruktioner med personligt grundled og infinitiv. |
| Efter modalverbum | Nogle konstruktioner med fx `kan`, `skal`, `vil` og bøjet verbum. |
| Køn og tal | Artikel/navneord, tillægsord/navneord og udvalgte omsagn. |
| Gentagelser | Udvalgte funktionsord, med undtagelser for forskellige syntaktiske roller. |
| Stil/terminologi | Valgfrie Vale-pakker; ordvalg er ikke automatisk en sproglig fejl. |

De otte `Danish.*`-kontroller er eksperimentelle og aktive i denne prototypes
hybridmotor. De ti oprindelige Vale-regler har status `starter`. Ingen af
disse statusser betyder eksternt sprogfagligt godkendt. Se
[grammatikreglerne](grammar.md) og [Vale-regelkataloget](rules.md).

| Profil | Vale-regler oven i hybridmotoren |
| --- | --- |
| `dansk` (standard) | Fire præcise stavemønstre |
| `klart-sprog` | Desuden fire forslag til enklere formuleringer |
| `agentics` | Desuden to organisationsspecifikke termvalg |
| `experimental` | Kun nye Experimental-regler; hybridens grammatik er stadig aktiv |

`--checks grammar` kører kun grammatik; `--checks spelling` kører kun
stavning. `--engine vale` vælger den gamle Vale-motor uden sprogmodel;
der gælder Vales egen dokumentbehandling, og hybridens filtre/ordliste
anvendes ikke. Standardmotoren er `--engine hybrid`.

## Filer, undtagelser og integration

Angiv `.md`- og `.txt`-filer eksplicit eller via shell-glob. Standard input
læses som Markdown. Markdown-behandlingen bevarer kildepositioner og maskerer
kode, URL'er, linkdestinationer og strukturelle metadata. Titel og beskrivelse
kontrolleres. En enkel sproglig heuristik springer engelsktunge tekststykker
over; den er ikke fuld sproggenkendelse. MDX/JSX og alle Markdown-varianter
understøttes ikke. Grammatikreglerne arbejder ikke på tværs af linjer.

JSON indeholder regel-id, alvor, forklaring, `Match`, `Suggestions`, `Line`
og `Span` i originalteksten. Linjer/kolonner er 1-baserede, slutkolonnen
er inkluderet. Hybridens `Start`/`End` er Python-tegnpositioner med eksklusiv
slutposition, **ikke bytepositioner eller JavaScript UTF-16-positioner**.

Ukendte ord med svag evidens skjules normalt. Se dem eller tillad fagord lokalt:

```sh
bin/danish lint --include-uncertain artikel.md
bin/danish lint --dictionary vocabularies/development.txt artikel.md
bin/danish lint --checks grammar --fail-on warning artikel.md
```

En ordliste indeholder ét ord pr. linje; `#` indleder kommentarer. Ordlisten
er valgfri, og blogevalueringen bruger den ikke. Den undertrykker fund på
det angivne ord. Den engelske hjælpeordbog reducerer støj fra låneord, men
kan også skjule danske fejl, som tilfældigvis er engelske ord.

```md
<!-- vale Danish.ArticleGender = NO -->
Her står et tilsigtet citat med et workshop.
<!-- vale Danish.ArticleGender = YES -->
```

Brug `Danish` til at undtage alle hybridfund i en passage. Præcise Vale-id'er
som `Dansk.Interesse` kan også undtages. CLI'en har ingen automatisk rettelse.

Exitkoder: `0` = ingen fund på valgt fejlniveau, `1` = fund på/over niveauet,
`2` = værktøjs- eller inputproblem. Standard `--fail-on error` viser nye
advarsler uden at fejle CI; vælg `--fail-on warning` for at gøre det.
