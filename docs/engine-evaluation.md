# Motorvalg: historisk Python-afprøvning

Dette dokument beskriver Python-alphaen ved `a8bf898`. Beslutningen om at
beholde Python er erstattet af [Rust-porteringen](rust-migration.md).
Den aktive implementering findes i `src/` og kører uden Python.

Den oprindelige Vale-pakke fandt ingen fejl med standardprofilen i 179
blogindlæg. Terminologipakken gav 84 advarsler om `features`/`workflow`.
Det dokumenterede ikke nyttig dansk korrektur. Fire præcise stavemønstre
og stilregex var for smalt et fundament for brugerens mål.

Den nuværende prototype kombinerer eksisterende sprogresurser med
selvstændige, afgrænsede regler: Hunspell/Stavekontrolden til orddannelse,
COR til bøjningsdata, spaCy til dansk ordklasse-/syntaksanalyse og Vale til
simple mønstre og valgfri stil. Resultater og fejlkilder er gennemgået i
[evalueringen](evaluation.md).

Dansk er et kendt sprogteknologisk domæne, og vi kan genbruge data og
analysemodeller. Det svære er at vælge, hvornår en analyse giver tilstrækkelig
evidens til en advarsel. Modellen kan fortolke en fejl som en anden ordklasse,
mens en ordbog kan foreslå et ord med en helt forkert betydning. Flere regex
eller en omskrivning til Rust løser ikke det problem.

Behold Python til at afprøve regler og testgrundlag. Den samlede installation
er større end en enkel Rust-binær og er ikke en browserløsning. Rust/WASM,
editorintegration og distribution giver først mening, når vi har bevist
sproglig nytte på uafhængige tekster og valgt hvilke analyser der skal med.
LanguageTool indgår ikke i implementeringen; det danske modul er markeret
som ikke længere vedligeholdt og er ikke antaget at give bred dansk dækning.

Næste investering bør være et uafhængigt, menneskeligt annoteret testgrundlag
og sprogfaglig reviewkapacitet. Se evalueringens konkrete fortsæt/stop-kriterier.

Referencer: [COR](https://ordregister.dk/),
[Stavekontrolden](https://stavekontrolden.dk/),
[spaCy dansk](https://spacy.io/models/da),
[Vale](https://github.com/vale-cli/vale),
[LanguageTool dansk](https://github.com/languagetool-org/languagetool/tree/master/languagetool-language-modules/da),
[english-rs](https://github.com/Aaryan-Kapoor/english-rs).
