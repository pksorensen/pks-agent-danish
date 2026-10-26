// Render the actual Lottie player, so the README GIF matches the vector source.
// Optional build tools: playwright@1.58.2, lottie-web@5.13.0, ffmpeg on PATH.
const { chromium } = require('playwright');
const fs = require('node:fs');
const path = require('node:path');
const { spawn, spawnSync } = require('node:child_process');
const { once } = require('node:events');
const root = path.resolve(__dirname, '..');
const work = path.resolve(process.argv[2] || path.join(root, 'data/animation'));
const browserPath = process.argv[3];
(async () => {
  fs.mkdirSync(work, { recursive: true });
  const animation = JSON.parse(fs.readFileSync(path.join(root, 'docs/assets/danish-lint.json')));
  const browser = await chromium.launch({ headless: true, ...(browserPath ? {executablePath: browserPath} : {}) });
  try {
    const page = await browser.newPage({ viewport: {width: animation.w, height: animation.h}, deviceScaleFactor: 1 });
    const errors = [];
    page.on('pageerror', e => errors.push(e.message));
    await page.setContent('<html><style>html,body{margin:0;background:#f7f5ef}#scene{width:1000px;height:580px}</style><div id="scene"></div></html>');
    await page.addScriptTag({path: require.resolve('lottie-web/build/player/lottie.min.js')});
    await page.evaluate(data => {
      window.player = lottie.loadAnimation({container: document.getElementById('scene'), renderer:'svg', loop:false, autoplay:false, animationData:data});
      player.addEventListener('DOMLoaded', () => window.ready = true);
      player.addEventListener('data_failed', () => {throw new Error('Lottie data failed');});
    }, animation);
    await page.waitForFunction(() => window.ready);
    const encoder = spawn('ffmpeg', ['-y','-loglevel','error','-f','image2pipe','-framerate',String(animation.fr),'-i','pipe:0','-an','-c:v','libx264','-threads','2','-pix_fmt','yuv420p',path.join(work,'preview.mp4')], {stdio:['pipe','inherit','inherit']});
    const done = once(encoder, 'exit');
    for (let f = 0; f < animation.op; f++) {
      await page.evaluate(frame => player.goToAndStop(frame, true), f);
      const buffer = await page.screenshot({type:'png'});
      if ([0,45,100,170,215].includes(f)) fs.writeFileSync(path.join(work,`frame-${f}.png`),buffer);
      if (f === 170) fs.writeFileSync(path.join(root,'docs/assets/danish-lint-poster.png'),buffer);
      if (!encoder.stdin.write(buffer)) await once(encoder.stdin,'drain');
    }
    encoder.stdin.end();
    if ((await done)[0] !== 0) throw new Error('Video encoding failed');
    if (errors.length) throw new Error(errors.join('\n'));
    const gif = spawnSync('ffmpeg',['-y','-loglevel','error','-threads','2','-i',path.join(work,'preview.mp4'),'-filter_complex_threads','1','-filter_complex','fps=18,split[a][b];[a]palettegen=max_colors=128:stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=3','-loop','0',path.join(root,'docs/assets/danish-lint.gif')],{stdio:'inherit'});
    if (gif.status !== 0) throw new Error('GIF encoding failed');
    console.log('Rendered 240 Lottie frames; no browser errors. GIF and poster ready.');
  } finally { await browser.close(); }
})().catch(error => {console.error(error); process.exit(1);});
