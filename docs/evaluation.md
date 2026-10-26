# Python-prototypens evaluering (historisk)

Denne rapport gælder Python-alphaen på commit `a8bf898`, før Rust-porteringen.
Kommandoer og testtal nedenfor er historiske; brug [Rust-evalueringen](rust-migration.md) for den aktuelle version.

**Vurdering, 2026-10-26:** Fortsæt som en afgrænset dansk korrekturassistent.
Prototypen giver nu nyttige fund på rigtige tekster. Den er endnu ikke klar
til at være en generel dansk kvalitetsgaranti, og denne afprøvning beviser
ikke, at produktet kan opnå bred udbredelse.

## Resultat på bloggen

179 kanoniske `blog-posts/*/da.md` blev kontrolleret med samme funktion som
hybrid-CLI'en, standardprofilen `dansk`, uden lokal fagordsliste. Køretid
inklusive opstart og de 61 kvalitetseksempler: **94,76 sekunder** i det
aktuelle Linux-miljø. Ingen inputfiler er ændret; SHA-256 blev kontrolleret
før/efter og gemt for hver fil. Alle rapporterede positioner blev verificeret
mod originalteksten.

| Resultat | Antal |
| --- | ---: |
| Standardadvarsler | 72 |
| Filer med standardadvarsler | 49 |
| Forskellige kombinationer af regel, ord og forslag | 44 |
| Ukendte ord, kun synlige med `--include-uncertain` | 1.064 |
| Tidligere Vale-standardpakke på samme blogudvalg | 0 fund |

De 1.064 ukendte ord er **ikke 1.064 stavefejl**. De omfatter fagord,
navne og fejl, som motoren ikke har tilstrækkelig evidens til at klassificere.
Standardadvarslerne omfatter 24 stavefund, 20 artikel/køn-fund,
25 adjektivfund, 2 omsagnsfund og 1 modalverbumfund. Nutids-r og gentagelser
gav ingen fund i denne kørsel, selv om nogle af deres testeksempler detekteres.

Jeg har gennemgået alle 72 advarsler i deres kontekst:

| Min udviklerklassifikation | Antal |
| --- | ---: |
| Nyttig lokal rettelse | 50 |
| Nyttig delrettelse, men hele udtrykket skal rettes | 2 |
| Kræver sprogfaglig/terminologisk vurdering | 18 |
| Forkert rettelsesforslag | 2 |

Denne vurdering er lavet af **samme agent som implementerede reglerne**.
De 18 uafklarede fund tælles ikke som korrekte. Gentagne forekomster tæller
hver for sig, fx optræder `plejedede` flere steder. Se [alle 72 vurderinger](blog-findings.md).
Der er hverken målt uafhængig præcision eller recall: Vi har ikke et facit
for samtlige fejl i de 179 tekster. Bloggen er brugt under udviklingen, så
resultatet er heller ikke et resultat på tilbageholdt testmateriale.

## Eksempler på konkret nytte

| Original tekst | Relevant rettelse | Kilde |
| --- | --- | --- |
| `Modellerne er dygtig` | `Modellerne er dygtige` | `987-prompts-milestone/da.md:31` |
| `Det er et workshop` | `Det er en workshop` | `have-i-coded-myself-out-of-a-job/da.md:63` |
| `et rigtig produkt` | `et rigtigt produkt` | `build-over-buy/da.md:48` |
| `vinduet var færdig` | `vinduet var færdigt` | `motion-hype-new-director/da.md:222` |
| `utrodeligt` | `utroligt` | `have-i-coded-myself-out-of-a-job/da.md:63` |
| `brusebaddet` | `brusebadet` | `verbos-92-claude-code-agent-swarms/da.md:65` |

Dette er en forbedring i faktisk korrekturværdi i forhold til den første
pakkes fire stavemønstre. Det kommer fra genbrug af ordbog, bøjningsdata
og sætningsanalyse; antallet af håndskrevne regex er ikke forklaringen.

## Fejl, som stadig viser produktets grænser

- `et ask` om en engelsk *anmodning* bliver foreslået ændret til `en ask`,
  fordi motoren analyserer det danske trænavn. En korrekt ordklasse og et
  korrekt ordbogsopslag kan stadig give en forkert advarsel.
- `gøre det sjovt processen` er fejlbehæftet, men forslaget `sjove` løser
  ikke problemet. En relevant omskrivning er `gøre processen sjov`.
- `en ærlig argument` udløser `ærlig → ærligt`, men den samlede rettelse
  er `et ærligt argument`. Brugeren skal kunne se hele konstruktionen.
- Engelske tekniske ord som `feature`, `download` og `benchmark` giver
  fund, som kræver en vurdering af betydning, norm og faktisk kønsbrug.
- `Hun gav et ærlig svar` overses. COR analyserer `svar` som flere
  ordklasser og både ental og flertal; den forsigtige regel afstår.

Den generelle staveheuristik springer ord med stort begyndelsesbogstav
over for at mindske fejl på produktnavne. Engelskordbogen dæmper støj,
men skjuler også nogle fejl. Disse valg prioriterer få forkerte advarsler
og begrænser samtidig dækningen.

## Test og installation

De 61 selvskrevne kvalitetscases indeholder 21 fejlsætninger og 40 korrekte
kontroller. **20 af 21 fejlsætninger og alle 40 kontroller** opfylder de
forventede fund og rettelser. Det oversete `ærlig svar` er bevaret i
materialet og synligt som en `expectedFailure` i regressionstestene.

13 unittest-metoder kørt: 12 bestået og 1 kendt forventet fejl. De omfatter
84 oprindelige Vale-regeleksempler, kvalitetseksempler, ekstra kontroller
fra blogafprøvningen, kode/metadata, kildepositioner, undtagelser, ordliste,
JSON og exitkoder. Genereret regelkatalog er kontrolleret. Installationsscriptet
er kørt med de fastlåste afhængigheder og verificerede data, og modellen
indlæses. Det eksisterende isolerede miljø blev genbrugt; et helt nyt,
tomt miljø og GitHub Actions er ikke afprøvet her. CI er opdateret til
Python 3.12, Hunspell og begge testsuiter.

## Hvad kræves for at blive et værktøj, folk bliver ved med at bruge?

Der er nok evidens til at finansiere en **afgrænset valideringsrunde**, ikke
til at love fuld dansk grammatik. Mit forslag er 2–4 ugers arbejde med en
udvikler og løbende adgang til en dansksproglig fagperson. Det er et
planlægningsestimat, ikke dokumenteret leveringstid. Målet er:

1. Et licensafklaret, menneskeligt annoteret korpus på 300–500 tekststykker
   fra flere genrer. Opdel efter kilde/forfatter; sæt en del helt til side
   til evaluering. Markér også alle fejl i udvalgte hele tekster, så vi kan
   måle, hvad motoren overser. En anden reviewer afgør tvivlstilfælde.
2. Vælg 5–10 hyppige fejltyper og mål hver regel særskilt. Ret de dokumenterede
   forkerte forslag, og test hele rettelsen i kontekst. Ny kode skal have
   fejl, modeksempler, kilde og en gennemgang af ændrede korpusfund.
3. Foreslået udgivelseskrav: mindst 95 % korrekte standardadvarsler på det
   tilbageholdte materiale, med tilstrækkeligt mange fund pr. regel til at
   resultatet er meningsfuldt. Rapportér usikkerhed og recall separat.
   Som første dækningsmål: fang mindst halvdelen af de annoterede fejl i
   de valgte fejltyper. Tallene er mål, ikke opnåede resultater.
4. Lad 5–10 skribenter bruge den på egne tekster gennem flere skriveforløb.
   Registrér accepterede/afviste forslag og om brugerne faktisk vælger at
   bruge den igen. Ingen af disse brugerundersøgelser er udført endnu.

Hvis vi kun kan få høj præcision ved at tie om næsten alt, eller brugerne
oplever for meget støj, skal vi indsnævre produktet eller stoppe ambitionen
om en bred korrekturassistent. Det er et reelt stopkriterium.

Community kan levere eksempler, kilder og modargumenter. Der skal stadig
være nogen, som tager ansvar for sproglig kvalitet. Derfor beskriver
[bidragsvejledningen](../CONTRIBUTING.md) både bidrag uden kode, YAML-regler,
syntaktiske regler og sprogfagligt/teknisk review. Repositoryet og issue-formularer er nu offentlige. Flere sprogfaglige
reviewers, editorintegration og pakkedistribution er stadig næste produktarbejde. En brugervenlig editor/agent-integration er nødvendig for
udbredelse, men kan ikke erstatte kvaliteten af advarslerne.

## Reproduktion

I det oprindelige Agentics-workspace (blogkorpusset er ikke bundlet med dette repository):

```sh
projects/pks-agent-danish/bin/danish lint blog-posts/987-prompts-milestone/da.md

data/pks-agent-danish/venv/bin/python projects/pks-agent-danish/scripts/evaluate.py \
  --blog-root blog-posts \
  --output data/pks-agent-danish/second-attempt/reproduced.json
```

Den gemte kørsel ligger i
`data/pks-agent-danish/second-attempt/final-evaluation.json` med filhashes,
alle diagnostikker og kvalitetseksemplernes resultater. Min vurdering af
hvert standardfund ligger i `reviewed-findings.json` i samme mappe; den
indeholder også hash af den evalueringsfil, som vurderingen gælder.
Rådata ligger på workspace-disken og indgår ikke i produktets kildekode.

I et selvstændigt checkout kan du køre de medfølgende kvalitetseksempler:

```sh
python3 scripts/test.py
data/pks-agent-danish/venv/bin/python scripts/evaluate.py \
  --output data/quality-evaluation.json
```

Tilføj `--blog-root /sti/til/dit-korpus` for egne mapper med `*/da.md`.
De oprindelige rårapporter og filhashes ligger kun i udviklingsworkspacet;
[tabellen over alle fund](blog-findings.md) er med i dette repository og
linker til de offentlige blogindlæg.
