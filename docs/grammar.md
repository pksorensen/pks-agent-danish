# Grammatik og stavning i hybridmotoren

Reglerne bruger spaCy `da_core_news_sm` 3.8.0 til ordklasser og
sætningsrelationer. COR 1.5.1.0 giver mulige analyser af den faktisk skrevne
ordform og bøjningsformer fra samme lemma. Modellen bruges til analyse,
ikke til at generere omskrivninger. Der kaldes ingen ekstern sprogmodel.

| Regel | Udløsende konstruktion | Centrale begrænsninger |
| --- | --- | --- |
| `Danish.FiniteVerb` | Personligt grundled + infinitiv, fx `Hun reparere cyklen` | Afstår ved hjælpeverber, tvetydigt `de` og mange usikre analyser. Finder ikke alle manglende nutids-r. |
| `Danish.AfterModal` | Grundled + modalverbum + nutidsform | Kræver afgrænset relation; finder ikke alle ordstillinger. |
| `Danish.PredicateAgreement` | Grundled + kopula + tillægsord | Afstår bl.a. ved passiver, masseord, præpositionsled og `klar over`/`vant til`. |
| `Danish.ArticleGender` | Artikel direkte foran entydigt navneord | Låneord kan have en anden betydning/kønsbrug end den danske ordbogsanalyse. |
| `Danish.AdjectiveAgreement` | Tillægsord koblet til et efterfølgende navneord | Køn, tal, bestemthed og possessiver i korte konstruktioner. Tvetydige ordformer giver huller. |
| `Danish.RepeatedWord` | Gentaget funktionsord | Afstår ved nogle forskellige ordklasser; kan dermed også overse reelle dubletter. |
| `Danish.Spelling` | Ukendt ord + snævert rettelsesforslag | Søgehjælpsdata eller intern indsættelse/sletning med fælles begyndelse/slutning. En nær stavemåde beviser ikke den tilsigtede betydning. |
| `Danish.UnknownWord` | Ukendt ord uden tilstrækkelig evidens | Kun med `--include-uncertain`; har niveau `suggestion`, ikke `warning`. |

Kilde til bøjningsdata: [Det Centrale Ordregister](https://ordregister.dk/).
Normlagene N og K anvendes. [COR.SOEGEHJAELP](https://ordregister.dk/doc/COR.SOEGEHJAELP.html)
er søgehjælp, **ikke en liste over autoriserede rettelser**. Gyldige ordformer
og ordbogsaccepterede varianter frasorteres først; selv da kan rettelser være
forkerte. Stavekontroldens produktive orddannelse supplerer COR.

De nuværende konstruktioner er projektets selvstændige implementering.
En autoritativ ordbog gør ikke parserens syntaktiske analyse autoritativ.
Beskeder er forslag, og flere fund kan kræve en samlet rettelse: ved
`en ærlig argument` er `ærligt` kun en del af `et ærligt argument`.

## Kendte huller

`Hun gav et ærlig svar` overses: `svar` har flere ordklasser og både ental
og flertal i COR. Der er ingen fuld kommakontrol, ordstillingskontrol,
kontrol af sammensatte ord eller semantisk vurdering af oversættelsesdansk.
Korrekte sætninger med fremmedord kan fejlmarkeres. Titler, citater og
ufuldstændige sætninger kan give analysefejl. Store begyndelsesbogstaver
udelukkes fra den generelle staveheuristik for at skåne egennavne; det
koster dækning. De præcise Vale-staveregler er fortsat case-insensitive.

## Tilføj en grammatikregel

1. Beskriv det konkrete problem, normkilden, den nødvendige sproglige
   relation og mindst én situation, hvor reglen skal afstå.
2. Tilføj fejl og korrekte modeksempler i `tests/quality.json`: regel-id,
   præcis markeret ordform og forventet rettelsesforslag. Brug egne eller
   licensafklarede eksempler. Samme emne og ordforråd skal optræde i begge grupper.
3. Implementér i `checker.py` med COR-analyser og relevante syntaksrelationer.
   Brug `emit` til diagnostik og kildepositioner. Brug aldrig en filsti eller
   blogtitel som en betingelse for en sproglig regel.
4. Registrér regel-id i `RULES`, udvid tabellen her og kør regressionstestene.
   Nye grammatikregler er endnu Python-kode; YAML alene kan ikke udtrykke
   den nødvendige syntaks. Dette er en større bidragsopgave end en stilregel.
5. Kør `scripts/evaluate.py` og gennemgå samtlige ændrede advarsler, også
   fjernede fund. Et højere antal fund er ikke i sig selv en forbedring.
6. Indsend kilder, undtagelser, positive/negative eksempler og fundændringen
   til sprogfagligt og teknisk review. En regel bliver ikke stabil alene
   ved at bestå dens egne eksempler.

En bruger skal kunne foreslå en regel ved kun at indsende en sætning og en
forklaring. Vedligeholderne omsætter bidraget til kode, test og dokumentation.
