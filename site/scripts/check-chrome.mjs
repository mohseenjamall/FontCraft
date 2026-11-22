// Verifies the page chrome (header, ruler, hero) shares one surface colour and stays legible on it.
import { readFileSync } from 'node:fs';

const read = (p) => readFileSync(new URL(`../src/${p}`, import.meta.url), 'utf8');
const css = read('styles/global.css');
const layout = read('layouts/Layout.astro');
const hero = read('components/Hero.astro');
const ruler = read('components/Ruler.astro');
const errors = [];
const need = (cond, msg) => { if (!cond) errors.push(msg); };

const cobalt = /--cobalt:\s*(#[0-9a-f]{6})/i.exec(css)?.[1];
need(cobalt, 'global.css defines no --cobalt');

// One source of truth: header and hero both read --cobalt, so they cannot drift apart.
need(/\.hero\s*\{[^}]*background:\s*var\(--cobalt\)/s.test(hero), 'hero background is not var(--cobalt)');
need(/data-on-hero\]\)\s*\.site-header\s*\{[^}]*background:\s*var\(--cobalt\)/s.test(layout), 'header over the hero is not var(--cobalt)');
need(/<html[^>]*data-on-hero/.test(layout), 'html starts without data-on-hero (header would flash dark)');
need(/html\[data-on-hero\]\s+\.ruler-bg\s*\{[^}]*background:\s*transparent/s.test(ruler), 'ruler strip is not transparent over the hero');

const hex = (h) => [1, 3, 5].map((i) => parseInt(h.slice(i, i + 2), 16));
const lum = ([r, g, b]) => 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
const lin = (v) => { v /= 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; };
const over = (fg, a, bg) => fg.map((c, i) => Math.round(c * a + bg[i] * (1 - a)));
const ratio = (a, b) => { const [x, y] = [lum(a), lum(b)].sort((p, q) => q - p); return (x + 0.05) / (y + 0.05); };

if (cobalt) {
  const bg = hex(cobalt);
  const white = [255, 255, 255];
  const tests = [
    ['label #fff', white, 4.5],
    ['nav links white 85%', over(white, 0.85, bg), 4.5],
    ['meta white 78%', over(white, 0.78, bg), 4.5],
    ['sub #e8eeff', hex('#e8eeff'), 4.5],
    ['headline #b9c9ff (large)', hex('#b9c9ff'), 3],
    ['headline #fff (large)', white, 3],
    ['button text #0b0b0c on white', null, 4.5],
  ];
  for (const [name, fg, min] of tests) {
    const r = fg ? ratio(fg, bg) : ratio(hex('#0b0b0c'), white);
    need(r >= min, `${name}: contrast ${r.toFixed(2)} < ${min}`);
  }
}

if (errors.length) { console.error(errors.join('\n')); process.exit(1); }
console.log('chrome check passed');
