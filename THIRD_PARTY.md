# Licenser og kilder

Projektets egen Rust-kode, regler og dokumentation er MIT. Ordbøger,
modeller og afhængigheder har egne licenser. De skal bevares ved distribution.
`Cargo.lock` fastlåser kildeversioner; `resources.lock.json` fastlåser
sprogdata med downloadadresse, størrelse og SHA-256.

## Indbygget analysemodel

Vores `models/danish-ddt.udpipe` er trænet på UD Danish DDT ved commit
`5880293fa36da950fa833fe0b52cb7d21882c481`. Model og træningsmateriale er
**CC BY-SA 4.0**. Modellen er en selvstændig træningsadaptation fra dette
projekt; der er ikke anvendt en prætrænet model med NC-begrænsning.
Se [modelkort og attribution](models/README.md) og
[licensteksten](models/LICENSE-DDT.txt). Disse medfølger native udgivelser.
Modellen må ikke ommærkes som MIT.

## Native biblioteker

- **UDPipe**, UFAL / Charles University, **MPL 2.0**, via `udpipe-rs` 0.2.0
  (MIT/Apache-2.0). Det oprindelige C++-bibliotek indbygges statisk;
  projektet har ikke ændret bibliotekets kildekode. Kildepakken kan hentes på
  https://crates.io/crates/udpipe-rs/0.2.0 .
  Licensmeddelelser findes under `licenses/` og i kildepakkens `vendor/udpipe/`.
- **Spellbook 0.4.2**, **MPL 2.0**, en Rust-implementering af Hunspell-formatet:
  https://crates.io/crates/spellbook/0.4.2 . Bibliotekets kildekode er uændret.
  Adapteren normaliserer numeriske flag/litterale skråstreger ved indlæsning
  af Stavekontroldens data; de downloadede originalfiler ændres ikke.
- Øvrige Rust-afhængigheder følger deres individuelle MIT, Apache, BSD
  eller kompatible licenser. Præcise versioner står i `Cargo.lock`;
  [licensmeddelelser fra kildepakkerne](licenses/dependencies.md) medfølger
  release-arkiverne. Opdatér samlingen, når afhængigheder ændres.

MPL-kildetekst for indbyggede biblioteker er tilgængelig gennem de angivne
kildepakker og versioner. Release-arkiver indeholder licensmeddelelserne.
Der følger hverken Python, spaCy, en Vale-binær eller et separat Hunspell-
bibliotek med Rust-versionen.

## Sprogdata hentet af `danish setup`

- **COR 1.5.1.0 og COR.SOEGEHJAELP 1.0**, Dansk Sprognævn, **CC0**:
  https://ordregister.dk/ og https://ordregister.dk/doc/COR.SOEGEHJAELP.html .
  Kun normlag N og K anvendes. Søgehjælp er ikke et facit for rettelser.
- **Stavekontrolden 2.9.101 (2026-06-07)**, Foreningen for frit tilgængelige
  sprogværktøjer, baseret på data fra Det Danske Sprog- og Litteraturselskab.
  Upstream tilbyder **GPL 2.0 / LGPL 2.1 / MPL 1.1**.
  `README_da_DK.txt` og licensmeddelelser i `.aff` medfølger download.
  Dette projekt distribuerer ikke ordbøgerne som MIT.
- **SCOWL en_US 2020.12.07**, engelsk hjælpeordbog fra LibreOffice dictionaries.
  Licenser og ophavsret til delkomponenterne står i de downloadede
  `README_en_US.txt` og `license.txt`. Bevar begge ved distribution.

Ordbøgerne hentes fra LibreOffice dictionaries commit
`cdaeab266e35c0f1ce367629c7f2079c64fe9351`. Se lockfilen for hver fil.

## Animation og skrifttype

Lottie-animation og render-script er selvstændigt skrevet og MIT-licenseret.
**Geist**, Copyright 2024 The Geist Project Authors, er **SIL OFL 1.1**.
Font, licens og forfatterliste ligger under `docs/assets/fonts/`. Fonten er
uændret; Lottie-filen bruger eksporterede bogstavkonturer.
https://github.com/vercel/geist-font

De valgfrie render-værktøjer er **lottie-web 5.13.0** (MIT) og **Playwright
1.58.2** (Apache-2.0); de installeres separat. FFmpeg leveres af systemet.
De er ikke afhængigheder for kompilering eller brug af dansk-linteren.

## Inspiration og tidligere prototype

[english-rs](https://github.com/Aaryan-Kapoor/english-rs) er inspiration;
ingen kode derfra er kopieret. YAML-reglerne bruger en afgrænset del af
Vale-formatet, men udføres nu i Rust. Den tidligere Python/spaCy/Vale-version
og dens særskilte licensnoter findes i historikken på commit `a8bf898`.
