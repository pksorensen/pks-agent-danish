#!/usr/bin/env python3
"""Build the self-contained Lottie README illustration. Requires fonttools 4.63.0."""
import json
from pathlib import Path
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont
from fontTools.pens.basePen import BasePen
from fontTools.pens.transformPen import TransformPen

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'docs/assets'
FONT = TTFont(OUT / 'fonts/Geist.ttf')
if 'fvar' in FONT:
    FONT = instantiateVariableFont(FONT, {'wght': 500}, inplace=False)
GLYPHS = FONT.getGlyphSet()
CMAP = FONT.getBestCmap()
UPM = FONT['head'].unitsPerEm
FPS, END = 24, 240
INK, MUTED = '#24283d', '#71758b'
PURPLE, PAPER, GREEN, AMBER = '#6854d8', '#f7f5ef', '#168367', '#c78a37'
layers = []


def prop(value): return {'a': 0, 'k': value}
def color(value): return [int(value[i:i+2], 16)/255 for i in (1, 3, 5)] + [1]
def anim(points):
    return {'a': 1, 'k': [{'t': t, 's': v if isinstance(v, list) else [v],
                          'i': {'x': [.65], 'y': [1]}, 'o': {'x': [.25], 'y': [0]}}
                         for t, v in points]}
def opacity(start, end=None):
    points = [(0, 0), (start, 0), (start+7, 100)] if start else [(0, 100)]
    if end is not None: points += [(end, 100), (end+6, 0)]
    return anim(points)
def fill(c): return {'ty': 'fl', 'c': prop(color(c)), 'o': prop(100), 'r': 1}
def layer(name, shapes, pos=(0, 0), alpha=None, scale=None):
    item = {'ddd': 0, 'ind': len(layers)+1, 'ty': 4, 'nm': name, 'sr': 1,
            'ks': {'o': alpha or prop(100), 'r': prop(0),
                   'p': pos if isinstance(pos, dict) else prop([*pos, 0]),
                   'a': prop([0, 0, 0]), 's': scale or prop([100, 100, 100])},
            'ao': 0, 'shapes': shapes, 'ip': 0, 'op': END, 'st': 0, 'bm': 0}
    layers.append(item)
    return item

def box(name, x, y, w, h, c, radius=12, alpha=None):
    return layer(name, [{'ty': 'rc', 'd': 1, 's': prop([w, h]), 'p': prop([0, 0]), 'r': prop(radius)}, fill(c)], (x+w/2, y+h/2), alpha)

def ellipse(name, x, y, w, h, c, alpha=None):
    return layer(name, [{'ty': 'el', 'd': 1, 's': prop([w, h]), 'p': prop([0, 0])}, fill(c)], (x, y), alpha)

class Outline(BasePen):
    def __init__(self):
        super().__init__(GLYPHS)
        self.contours = []
        self.current = None
    def _moveTo(self, p):
        self.current = {'v': [list(p)], 'i': [[0, 0]], 'o': [[0, 0]], 'c': True}
    def _lineTo(self, p):
        self.current['v'].append(list(p)); self.current['i'].append([0,0]); self.current['o'].append([0,0])
    def _curveToOne(self, p1, p2, p3):
        c = self.current
        v = c['v'][-1]
        c['o'][-1] = [p1[0]-v[0], p1[1]-v[1]]
        c['v'].append(list(p3)); c['i'].append([p2[0]-p3[0],p2[1]-p3[1]]); c['o'].append([0,0])
    def _closePath(self):
        c = self.current
        if c['v'][-1] == c['v'][0] and len(c['v']) > 1:
            c['i'][0] = c['i'][-1]
            for key in ('v','i','o'): c[key].pop()
        self.contours.append(c)
        self.current = None
    def _endPath(self): self._closePath()

def width(text, size): return sum(GLYPHS[CMAP[ord(c)]].width for c in text) * size / UPM

def text(name, value, x, baseline, size, c=INK, alpha=None):
    shapes, offset = [], 0
    for char in value:
        pen = Outline()
        GLYPHS[CMAP[ord(char)]].draw(TransformPen(pen, (size/UPM, 0, 0, -size/UPM, offset, 0)))
        for contour in pen.contours:
            for key in ('v','i','o'): contour[key] = [[round(n, 3) for n in p] for p in contour[key]]
            shapes.append({'ty': 'sh', 'ks': prop(contour)})
        offset += width(char, size)
    return layer(name, [*shapes, fill(c)], (x, baseline), alpha)

def path(name, points, c, thickness=2, alpha=None):
    shape = {'v': points, 'i': [[0,0]]*len(points), 'o': [[0,0]]*len(points), 'c': False}
    return layer(name, [{'ty':'sh','ks':prop(shape)}, {'ty':'st','c':prop(color(c)), 'o':prop(100), 'w':prop(thickness),'lc':2,'lj':2}], alpha=alpha)

box('Warm paper',0,0,1000,580,PAPER,24)
ellipse('Lavender atmosphere',937,43,210,210,'#eae5f7')
ellipse('Green atmosphere',26,567,188,188,'#e1eee6')
for i in range(11):
    ellipse('Quiet grid',760+i*18,179,2,2,'#d5d1e2')
box('Language mark',44,38,39,32,PURPLE,9)
text('Language glyphs','æ',53,61,23,'#ffffff')
text('Repository','pks-agent-danish',96,60,18)
box('Open-source pill',789,39,165,30,'#ede9e0',15)
ellipse('Status dot',805,54,6,6,GREEN)
text('Open source','OPEN SOURCE',818,59,12,MUTED)
text('Headline','Dansk, med omtanke.',44,139,49)
text('Subtitle','Små sproglige fund. Plads til din stemme.',46,173,19,MUTED)
# Layered document shadow and frame.
box('Card shadow',53,213,894,303,'#e7e3dc',18)
box('Document',46,204,908,304,'#ffffff',18)
box('Card chrome',46,204,908,51,'#f0eef8',18)
box('Chrome square lower corners',46,235,908,20,'#f0eef8',0)
for i,c in enumerate(('#d4c9ed','#e0d5b5','#bbd9cb')): ellipse('Window dot',68+i*15,229,7,7,c)
text('Document name','kladde.md',121,234,13,MUTED)
text('Mode','DANSK / FORSLAG',789,234,11,MUTED)
rows = [
    ('Modellerne er ', 'dygtig', 'dygtige', 'KØN & TAL', 296, 87),
    ('Det er ', 'et workshop', 'en workshop', 'ARTIKEL', 355, 115),
    ('Et ', 'rigtig produkt', 'rigtigt produkt', 'BØJNING', 414, 143),
]
for index, (prefix, bad, good, label, y, frame) in enumerate(rows):
    x = 103 + width(prefix,27)
    text('Line number', f'0{index+1}',66,y,12,'#b1adbf')
    text('Sentence prefix',prefix,103,y,27)
    text('Original word',bad+'.',x,y,27,INK,opacity(0,frame))
    # A dotted marker draws attention without representing a hard error.
    w=width(bad,27)
    underline=path('Amber underline',[[x,y+8],[x+w,y+8]],AMBER,2,opacity(22+index*9,frame))
    box('Suggestion highlight',x-5,y-28,width(good,27)+11,37,'#e6f4ee',7,opacity(frame+3))
    text('Suggested word',good+'.',x,y,27,GREEN,opacity(frame+5))
    text('Rule category',label,790,y-1,10,MUTED,opacity(22+index*9,frame))
    ellipse('Accepted circle',916,y-9,20,20,'#ddf0e6',opacity(frame+5))
    path('Check',[[911,y-9],[915,y-5],[922,y-13]],GREEN,2,opacity(frame+8))
path('Footer divider',[[67,440],[932,440]],'#eeedf3',1)
text('Initial status','3 steder at kigge nærmere.',69,479,15,MUTED,opacity(0,150))
text('Final status','Forslag vist. Du bestemmer.',69,479,15,GREEN,opacity(155))
box('Button',743,453,189,38,PURPLE,10)
text('Button label','Vis forslag',802,478,14,'#ffffff')
path('Button sparkle',[[763,462],[763,480]],'#ddd6ff',1.5)
path('Button sparkle',[[754,471],[772,471]],'#ddd6ff',1.5)
# Cursor visits the suggestion button; it does not suggest automatic file edits.
cursor={'v':[[0,0],[0,21],[6,16],[11,26],[16,23],[10,14],[19,13]],'i':[[0,0]]*7,'o':[[0,0]]*7,'c':True}
layer('Pointer', [{'ty':'sh','ks':prop(cursor)},fill(INK),{'ty':'st','c':prop(color('#ffffff')),'o':prop(100),'w':prop(2),'lj':2}],
      pos=anim([(0,[929,526,0]),(47,[929,526,0]),(66,[881,478,0]),(77,[881,478,0]),(88,[918,520,0])]),
      alpha=opacity(38,99), scale=anim([(0,[100,100,100]),(69,[100,100,100]),(73,[85,85,100]),(77,[100,100,100])]))
text('Footer principle','LOKALT. FORKLARLIGT. ÅBENT.',46,550,12,MUTED)
text('Community invitation','Bedre dansk bygger vi sammen.',668,550,16,PURPLE)
# Brief closing hold, then a soft paper wipe makes the repeat intentional.
box('Loop transition',0,0,1000,580,PAPER,24,anim([(0,0),(218,0),(231,100),(239,100)]))
scene = {'v':'5.12.2','fr':FPS,'ip':0,'op':END,'w':1000,'h':580,'nm':'Dansk, med omtanke',
         'ddd':0,'assets':[],'layers':list(reversed(layers)),
         'markers':[{'tm':0,'cm':'Original tekst','dr':80},{'tm':87,'cm':'Forslag','dr':76},{'tm':170,'cm':'Du bestemmer','dr':48}]}
OUT.mkdir(exist_ok=True)
(OUT/'danish-lint.json').write_text(json.dumps(scene,ensure_ascii=False,separators=(',',':'))+'\n')
print(f'Built {len(layers)} vector layers, {END/FPS:g}s, no external fonts or images')
