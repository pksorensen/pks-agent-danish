# Tredjepartsnoter

## Vale 3.25.1

Installeres særskilt af `scripts/install_vale.py`. Arkivernes SHA-256 er
fastlåst i scriptet. Binæren og arkiverne indgår ikke i projektets kildekode.

- Kilde og udgivelse: https://github.com/vale-cli/vale/releases/tag/v3.25.1
- Licens: MIT, https://github.com/vale-cli/vale/blob/v3.25.1/LICENSE

Ved eventuel distribution af Vale sammen med projektet skal Vales
licensmeddelelse og relevante tredjepartsmeddelelser medfølge.

## Sproglige kilder og inspiration

Retskrivningsordbogen henvises til som normkilde. Regelbeskrivelser og
testtekster er skrevet selvstændigt til projektet. De downloadede sprogresurser
er beskrevet nedenfor.

english-rs er inspiration til grænsefladen og tilgangen. Der er ikke kopieret
kode derfra. LanguageTool nævnes som en mulig fremtidig sammenligning;
ingen LanguageTool-kode eller regler indgår.

## Sprogdata og model — tilføjet i anden prototype

Den aktuelle installation henter følgende særskilte data.
De er **ikke** omfattet af projektets MIT-licens.

- **COR 1.5.1.0 og COR.SOEGEHJAELP 1.0**, Dansk Sprognævn:
  [ordregister.dk](https://ordregister.dk/) og
  [søgehjælpsdokumentation](https://ordregister.dk/doc/COR.SOEGEHJAELP.html).
  Udgivet under CC0. Originaldata opbevares uændret; motoren filtrerer normlag
  og slår bøjningsformer op under kørsel.
- **Stavekontrolden 2.9.101 (2026-06-07)**, © 2020 Foreningen for frit
  tilgængelige sprogværktøjer. Bygger på data fra Det Danske Sprog- og
  Litteraturselskab. Upstream tilbyder GPL 2.0, LGPL 2.1 og MPL 1.1.
  `README_da_DK.txt` følger med download. Der hentes kun Hunspell-filer,
  ikke synonymordbogen.
- **SCOWL en_US 2020.12.07**, engelsk hjælpeordbog fra LibreOffice dictionaries.
  De forskellige komponenters ophavsret og tilladelser står i de downloadede
  `README_en_US.txt` og `license.txt`; bevar begge ved distribution.
- **spaCy 3.8.7**, Explosion, MIT. **da_core_news_sm 3.8.0**, Explosion,
  [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
  Modellen bruges uændret og installeres særskilt som Python-pakke.
  Kilde: https://github.com/explosion/spacy-models/releases/tag/da_core_news_sm-3.8.0
  Modellens metadata krediterer UD Danish DDT v2.8 (Anders Johannsen,
  Héctor Martínez Alonso, Barbara Plank) og DaNE (Rasmus Hvingelby,
  Amalie B. Pauli, Maria Barrett, Christina Rosted, Lasse M. Lidegaard,
  Anders Søgaard), begge CC BY-SA 4.0. NER er deaktiveret i linteren.
- **Hunspell**, systemets delte bibliotek. Projektet bruger det gennem ctypes
  og distribuerer ikke biblioteket. Upstream tilbyder GPL/LGPL/MPL;
  se https://github.com/hunspell/hunspell for versionens licensvilkår.

Ordbøgerne er fastlåst til LibreOffice dictionaries commit
`cdaeab266e35c0f1ce367629c7f2079c64fe9351`. Præcise downloadadresser,
størrelser og SHA-256 fremgår af `resources.lock.json`.
Python-afhængigheder er fastlåst i `requirements.lock`; deres egne
licensmeddelelser følger pakkerne. Ved samlet distribution skal de relevante
licenser og meddelelser følge med; modellen må ikke ommærkes som MIT.

## README-animation

Animation og generator er selvstændigt skrevet til projektet. De bruger
**Geist**, Copyright 2024 The Geist Project Authors, under **SIL Open Font
License 1.1**. Den medfølgende font er ikke ændret; animationens bogstaver
er eksporteret som vektorkonturer. Font, `OFL.txt` og `AUTHORS.txt` ligger
under `docs/assets/fonts/`. Kilde: https://github.com/vercel/geist-font

Den valgfrie renderproces bruger **lottie-web 5.13.0** (MIT), **Playwright
1.58.2** (Apache-2.0) og **fonttools 4.63.0** (MIT). De installeres separat
og distribueres ikke som en del af dette repository. FFmpeg leveres af
brugerens system. GIF'en er renderet fra den medfølgende Lottie JSON.
