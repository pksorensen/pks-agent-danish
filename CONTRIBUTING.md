# Bidrag til dansk-linteren

En god regel hjælper skribenten og er tydelig om sine begrænsninger. Et
modeksempel, der fjerner en forkert advarsel, er et ligeværdigt bidrag.

## Bidrag uden kode

Brug skabelonen for [fejlagtige advarsler](docs/templates/false-positive.md)
eller [nye regler](docs/templates/new-rule.md). Angiv en kort, anonymiseret
sætning, regel-id, profil og hvad du forventer. Del kun tekst, du må dele.
Brug [GitHub Issues](https://github.com/pksorensen/pks-agent-danish/issues/new/choose).
Der er formularer til fejlagtige advarsler, nye regler og erfaringer med værktøjet.

## Fra observation til regel

Dette eksempel viser et stilforslag for frasen *i langt de fleste tilfælde*.
Det er en øvelse i bidragsprocessen, ikke en påstand om, at frasen er forkert.

1. **Afgræns formålet.** I korte hjælpetekster kan *oftest* være tilstrækkeligt.
   Den længere formulering kan samtidig understrege noget vigtigt. Derfor
   skal reglen være et eksperimentelt stilforslag, ikke en retskrivningsfejl.
2. **Opret filerne.** Kør fra projektmappen:

   ```sh
   python3 danish.py rule new Experimental.IFlesteTilfaelde
   ```

3. **Skriv mønstret** i `styles/Experimental/IFlesteTilfaelde.yml`:

   ```yaml
   extends: existence
   message: "Overvej 'oftest' i stedet for '%s', hvis nuancen kan undværes."
   level: warning
   ignorecase: true
   tokens:
     - 'i langt de fleste tilfælde'
   ```

   Brug Vales eksisterende [regeltyper](https://docs.vale.sh/checks/existence).
   `existence` passer til et forsigtigt stilforslag; `substitution` kan bruges
   til en entydig, dokumenteret stavemåde. Undgå regex, der kan ramme store
   dele af et dokument. Kontroller store bogstaver, æ/ø/å og ordgrænser.

4. **Udfyld regelbeskrivelsen** i `rules/Experimental.IFlesteTilfaelde.json`.
   Behold `id`, sæt `category` til `style` og `status` til `experimental`.
   Beskriv formål, begrænsninger og genre i `summary` og `explanation`.
   Angiv en reel kilde eller en begrundet stilbeslutning i en lokal Markdown-fil
   via `source` og `source_note`. Fjern alle pladsholdere. `autofix` er `false`.

5. **Tilføj eksempler** i JSON-filens `cases`. `matches` indeholder præcis de
   tekststykker, reglen skal markere, i rækkefølge. Et tomt array betyder, at
   teksten skal passere. Der kræves mindst to positive og tre negative eksempler:

   ```json
   [
     {"text": "Det virker i langt de fleste tilfælde.", "matches": ["i langt de fleste tilfælde"]},
     {"text": "I LANGT DE FLESTE TILFÆLDE virker det.", "matches": ["I LANGT DE FLESTE TILFÆLDE"]},
     {"text": "Det virker oftest.", "matches": []},
     {"text": "I nogle tilfælde kræves der hjælp.", "matches": []},
     {"text": "Mønstret er `i langt de fleste tilfælde`.", "matches": []}
   ]
   ```

   Tilføj også kodeblokke, lokale undtagelser og relevante grammatiske
   modeksempler. Fx skal en fremtidig gentagelsesregel lade *han har taget
   taget af* passere. Positive eksempler alene er ikke tilstrækkelige.

6. **Kør den rigtige motor og testene:**

   ```sh
   bin/danish lint --profile experimental min-tekst.md
   python3 tests/test_rules.py
   python3 scripts/catalog.py
   python3 scripts/catalog.py --check
   ```

   Testene opdager nye regelpar automatisk. De sammenligner fund, alvor og
   placering i originalteksten. Korpusfund vises pr. profil og fil. Hvis en
   korpusforventning ændres, skal hvert nyt eller fjernet fund forklares.

7. **Forbered review.** Beskriv problemet, kilden, kendte undtagelser, testene
   og eventuelle nye korpusfund. Brug skabelonen til [regelforslag](docs/templates/new-rule.md).

## Retskrivning og stil har forskellige krav

- **Retskrivning og grammatik:** henvis til den relevante regel eller det
  konkrete opslag hos fx Dansk Sprognævn. Eksemplerne skal også vise gyldige
  varianter. En kildehenvisning gør ikke et regex til en korrekt grammatikregel.
- **Stil:** navngiv målgruppe, genre og stilbeslutning. Formuler hjælpen som et
  forslag. Almindelige danske ord og låneord er ikke automatisk fejl.
- **Terminologi:** hører til en navngiven profil. En organisations foretrukne
  ordvalg bliver ikke en generel dansk norm.
- **Kommatering:** en fremtidig regel skal respektere det valgte kommasystem.
  Startkomma må ikke antages at være obligatorisk.

Skriv egne forklaringer og egne eksempler. Kopiér ikke ordbøger, korpora eller
regler fra andre værktøjer uden at afklare licensen. Ved korpusbidrag skal
oprindelse og brugsrettigheder stå ved materialet.
Bidrag til kode, regler, dokumentation og testmateriale gives under projektets
[MIT-licens](LICENSE). Der kræves ingen særskilt overdragelse af ophavsret.

## Review og udgivelser

Nye Vale-regler starter i `Experimental`, som aldrig indgår i standardprofilen.
Hybridmotorens nuværende grammatikregler er en særskilt prototype: de er
aktive som standard under afprøvningen, men er ikke stabile eller eksternt
godkendte. Fremtidige standardregler kræver samme review som beskrevet her.
Før en regel flyttes til en normal pakke, skal både en sprogfaglig og en
teknisk reviewer godkende afgrænsning, kilde, modeksempler og korpusvirkning.
Rollerne er reviewkrav; projektet har endnu ingen udpeget offentlig
maintainergruppe. Agentforslag tæller ikke som en sprogfaglig godkendelse.

De ti medfølgende regler har status `starter`: de er implementeret og testet,
men har endnu ikke været gennem et eksternt sprogfagligt review. De simple
staveregler er den foreløbige standardpakke. De må ikke markedsføres som
dokumentation for bred grammatisk korrekthed.

Efter første stabil udgivelse bevares regel-id'er. Ændringer i standardprofil,
alvor eller betydning kræver versionsnoter og en vurdering af kompatibilitet.
En eksperimentel regel får sit endelige id, før den bliver stabil. Lokale
undtagelser skal fortsat være mulige, og en fejlramt regel skal kunne slås fra.

Automatiske rettelser kræver en særskilt beslutning og tests for bl.a.
betydningsbevarelse, store bogstaver, overlappende fund og gentagen anvendelse.
Denne version tilbyder forklaringer og forslag, men skriver aldrig i teksten.

## Grammatik, testgrundlag og fælles ejerskab

Se [grammatikvejledningen](docs/grammar.md) for arbejde med COR, ordklasser
og syntaks. `tests/quality.json` rummer både fejl og korrekt dansk.
`tests/test_checker.py` kontrollerer desuden Markdown, kildepositioner,
ordlister, undtagelser og den samlede CLI. Kør med det installerede miljø:

```sh
python3 scripts/test.py
```

Bidrag kan starte på tre niveauer: en anonymiseret sætning uden kode,
en enkel YAML-regel eller en syntaktisk Python-regel. Vedligeholderne har
ansvar for at omsætte sproglige observationer til reproducerbare tests.
Det skal ikke være nødvendigt at kunne programmere for at forbedre sproget.

Den offentlige alpha søger sprogfaglige reviewers og tekniske bidragydere.
Kontakt og koordinering foregår i repositoryets issues.
En regel-PR skal vise nye/fjernede fund på udviklingskorpusset. Et separat
benchmark skal holdes ude af den løbende regeltilpasning og gennemgås af
mennesker; egne eksempler og agentvurderinger kan ikke erstatte det.
Stabile regler bør kunne trækkes tilbage, hvis indberetninger viser støj.
Versionsnoter skal forklare ændringer i reglernes betydning og standardvalg.
