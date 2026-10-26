# Regeloversigt

Genereret fra `rules/*.json` med `danish catalog`.
Redigér regelbeskrivelserne, og generér denne fil igen.

Startreglerne er afgrænsede og har endnu ikke eksternt sprogfagligt review.

## Agentics.Features

Brug dansk produktterminologi i kundevendt tekst.

Kategori: `terminology`. Status: `starter`. Automatisk rettelse: nej.

Agentics-profilen foretrækker 'funktioner' frem for 'features' i dansk kundevendt tekst. Det er en lokal terminologibeslutning og ingen generel påstand om korrekt dansk.

[Kilde](../docs/style-decisions.md#agentics). Lokal, dokumenteret stilbeslutning; ikke en officiel retskrivningsregel.

Undtagelser: Produktnavne, citater og engelske passager kan beholde ordet.

Se [testeksemplerne](../rules/Agentics.Features.json).

## Agentics.Workflow

Overvej 'arbejdsgang' i dansk kundevendt tekst.

Kategori: `terminology`. Status: `starter`. Automatisk rettelse: nej.

Agentics-profilen foretrækker 'arbejdsgang' frem for 'workflow'. Bøjning og artikel skal vurderes i sammenhæng. Reglen matcher kun grundformen.

[Kilde](../docs/style-decisions.md#agentics). Lokal, dokumenteret stilbeslutning; ikke en officiel retskrivningsregel.

Undtagelser: Teknisk dokumentation, produktnavne og citater kan beholde ordet.

Se [testeksemplerne](../rules/Agentics.Workflow.json).

## Dansk.Abonnement

Skriv 'abonnement' frem for 'abonnoment'.

Kategori: `spelling`. Status: `starter`. Automatisk rettelse: nej.

Reglen fanger den afgrænsede fejlstavning 'abonnoment' og foreslår 'abonnement'. Den er ikke en fuld stavekontrol og forsøger ikke at gætte andre bøjningsformer.

[Kilde](https://ro.dsn.dk/). Retskrivningsordbogen: opslag 'abonnement'. Reglen og eksemplerne er selvstændigt skrevet; ingen ordbogsdata er kopieret.

Undtagelser: Citater og omtale af selve fejlstavningen kan undtages lokalt. Egennavne kan kræve en lokal undtagelse.

Se [testeksemplerne](../rules/Dansk.Abonnement.json).

## Dansk.Desvaerre

Skriv 'desværre' frem for 'desvære'.

Kategori: `spelling`. Status: `starter`. Automatisk rettelse: nej.

Reglen fanger den afgrænsede fejlstavning 'desvære' og foreslår 'desværre'. Den er ikke en fuld stavekontrol og forsøger ikke at gætte andre bøjningsformer.

[Kilde](https://ro.dsn.dk/). Retskrivningsordbogen: opslag 'desværre'. Reglen og eksemplerne er selvstændigt skrevet; ingen ordbogsdata er kopieret.

Undtagelser: Citater og omtale af selve fejlstavningen kan undtages lokalt. Egennavne kan kræve en lokal undtagelse.

Se [testeksemplerne](../rules/Dansk.Desvaerre.json).

## Dansk.Interesse

Skriv 'interesse' frem for 'interresse'.

Kategori: `spelling`. Status: `starter`. Automatisk rettelse: nej.

Reglen fanger den afgrænsede fejlstavning 'interresse' og foreslår 'interesse'. Den er ikke en fuld stavekontrol og forsøger ikke at gætte andre bøjningsformer.

[Kilde](https://ro.dsn.dk/). Retskrivningsordbogen: opslag 'interesse'. Reglen og eksemplerne er selvstændigt skrevet; ingen ordbogsdata er kopieret.

Undtagelser: Citater og omtale af selve fejlstavningen kan undtages lokalt. Egennavne kan kræve en lokal undtagelse.

Se [testeksemplerne](../rules/Dansk.Interesse.json).

## Dansk.Konkurrence

Skriv 'konkurrence' frem for 'konkurence'.

Kategori: `spelling`. Status: `starter`. Automatisk rettelse: nej.

Reglen fanger den afgrænsede fejlstavning 'konkurence' og foreslår 'konkurrence'. Den er ikke en fuld stavekontrol og forsøger ikke at gætte andre bøjningsformer.

[Kilde](https://ro.dsn.dk/). Retskrivningsordbogen: opslag 'konkurrence'. Reglen og eksemplerne er selvstændigt skrevet; ingen ordbogsdata er kopieret.

Undtagelser: Citater og omtale af selve fejlstavningen kan undtages lokalt. Egennavne kan kræve en lokal undtagelse.

Se [testeksemplerne](../rules/Dansk.Konkurrence.json).

## KlartSprog.ForetageEnVurdering

Overvej et direkte udsagnsord.

Kategori: `style`. Status: `starter`. Automatisk rettelse: nej.

'Foretage en vurdering af planen' kan ofte skrives som 'vurdere planen'. En rettelse kræver også at fjerne eller tilpasse præpositionen. Reglen dækker kun infinitivfrasen.

[Kilde](../docs/style-decisions.md#klart-sprog). Lokal, dokumenteret stilbeslutning; ikke en officiel retskrivningsregel.

Undtagelser: En 'vurdering' kan betegne et særskilt fagligt dokument eller en formel proces.

Se [testeksemplerne](../rules/KlartSprog.ForetageEnVurdering.json).

## KlartSprog.FremsenderHermed

Overvej en enklere formulering ved fremsendelse.

Kategori: `style`. Status: `starter`. Automatisk rettelse: nej.

I almindelig information kan 'sender' ofte erstatte 'fremsender hermed'. Udtrykket er korrekt dansk og kan være passende i formel korrespondance. Reglen vurderer kun den præcise frase.

[Kilde](../docs/style-decisions.md#klart-sprog). Lokal, dokumenteret stilbeslutning; ikke en officiel retskrivningsregel.

Undtagelser: Formel eller juridisk korrespondance kan kræve denne stil.

Se [testeksemplerne](../rules/KlartSprog.FremsenderHermed.json).

## KlartSprog.MedHenblikPaa

Overvej at gøre formålet mere direkte.

Kategori: `style`. Status: `starter`. Automatisk rettelse: nej.

Frasen 'med henblik på' kan ofte omskrives til 'for at', men resten af sætningen skal tilpasses. 'Med henblik på salg' kan eksempelvis kræve 'for at sælge'. Frasen er grammatisk korrekt.

[Kilde](../docs/style-decisions.md#klart-sprog). Lokal, dokumenteret stilbeslutning; ikke en officiel retskrivningsregel.

Undtagelser: Bevar formuleringen, når den præciserer et fagligt eller juridisk formål.

Se [testeksemplerne](../rules/KlartSprog.MedHenblikPaa.json).

## KlartSprog.PaaNuvaerendeTidspunkt

Overvej 'nu' ved en henvisning til nutiden.

Kategori: `style`. Status: `starter`. Automatisk rettelse: nej.

'På nuværende tidspunkt' kan ofte forkortes til 'nu'. Den længere formulering kan dog understrege, at situationen forventes at ændre sig. Dette er et stilforslag.

[Kilde](../docs/style-decisions.md#klart-sprog). Lokal, dokumenteret stilbeslutning; ikke en officiel retskrivningsregel.

Undtagelser: Bevar formuleringen, når den tidslige afgrænsning er vigtig.

Se [testeksemplerne](../rules/KlartSprog.PaaNuvaerendeTidspunkt.json).
