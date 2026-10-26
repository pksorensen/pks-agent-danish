# Dansk, med omtanke — README-animation

En 10-sekunders Lottie-animation med danske fejlmarkeringer og forslag.
GitHub afspiller ikke Lottie direkte i README, så README viser en GIF,
der er renderet fra den samme Lottie-fil i en rigtig browser.

- [Lottie JSON](assets/danish-lint.json): vektorformer, 1000 × 580, 24 fps.
- [GIF](assets/danish-lint.gif): 18 fps, uendeligt loop, under 1 MB.
- [Statisk PNG](assets/danish-lint-poster.png): alternativ uden bevægelse.

Alle bogstaver er vektorkonturer i Lottie-filen. Den kræver ingen eksterne
skrifttyper, billeder eller netværksopslag ved afspilning. Den er afprøvet
med lottie-web 5.13.0 i Chromium; andre Lottie-renderere er ikke afprøvet.
Illustrationen viser forslag, ikke en eksisterende grafisk editor eller
automatisk rettelse af brugerens filer.

## Genskab animationen

Valgfrie designværktøjer, ikke en del af linterens runtime:

```sh
python3 -m venv data/animation-python
data/animation-python/bin/pip install fonttools==4.63.0
data/animation-python/bin/python scripts/build_animation.py

npm install --prefix data/animation-tools --no-save --package-lock=false \
  playwright@1.58.2 lottie-web@5.13.0
# Installer Chromium via Playwright, hvis du ikke allerede har den:
node data/animation-tools/node_modules/playwright/cli.js install chromium

NODE_PATH="$PWD/data/animation-tools/node_modules" \
  node scripts/render_animation.cjs data/animation-render
```

Rendereren kræver `ffmpeg` på PATH. En eksisterende Chromium-binær kan
angives som det næste argument efter outputmappen. Mellemvideo og enkelte
kontrolbilleder gemmes under `data/`; der skrives ikke hundredvis af frames
på disk. GIF og poster gemmes under `docs/assets/`.

Design og generator er selvstændigt skrevet til projektet og MIT-licenseret.
Geist-fonten er SIL OFL 1.1; licens og forfattere følger med i
[fonts/](assets/fonts/). lottie-web og Playwright installeres særskilt og
følger ikke med repositoryet.
