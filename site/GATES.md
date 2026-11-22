# Gates: FontCraft site chrome matches the hero

OWNS: site/src/layouts/**, site/src/components/Ruler.astro, site/src/components/Hero.astro, site/scripts/check-chrome.mjs, site/.impeccable.md, site/GATES.md

Scope: the header bar, the scroll ruler and the hero read as one cobalt surface at the top of the page, return to the dark theme after the hero, and every text colour on cobalt stays legible.

- [x] G1: header and hero share --cobalt, the ruler is transparent over the hero, and text on cobalt meets contrast
  CHECK: node scripts/check-chrome.mjs
  EXPECT: chrome check passed
  EVIDENCE: automatic-evidence=v1; definition-sha256=13e1b448282689ace108b6ea9bbe450b593069da91980cd1873657680faffdda; exit=0; EXPECT=matched; output-sha256=0fbec3ab585455678bf273984f82786a50478e6199c09f3fc0ea58681ba2e81e; output-bytes=20; shell=C:\WINDOWS\system32\cmd.exe; cwd=F:\Craft_Apps\fontbase\app\site; path=52c2527cc2a9/57 entries

- [x] G2: the site still builds into one static page
  CHECK: npm run build
  EXPECT: Complete!
  EVIDENCE: automatic-evidence=v1; definition-sha256=ab81a41a32b6b7b1f6f1376cc86da9a105dd409ebcf3a12d6a4d5b7659355bc3; exit=0; EXPECT=matched; output-sha256=5ccea1a6c452ef3b7962b206edbd8d9d335d214ca472fa3d485702eefa4472e1; output-bytes=2978; shell=C:\WINDOWS\system32\cmd.exe; cwd=F:\Craft_Apps\fontbase\app\site; path=52c2527cc2a9/57 entries

- [x] G3: in the browser at scrollY=0 the header and hero have the same computed background, and after scrolling past the hero the header is the dark theme again
  EVIDENCE: browser at 1440x900, scrollY=0: .site-header and .hero both rgb(26, 74, 196), ruler strip rgba(0,0,0,0), data-on-hero set. After scrolling to y=1300 (hero bottom at -337): data-on-hero removed, header target colour rgb(11, 11, 12), ruler strip back to dark translucent. Screenshot at top shows one continuous cobalt surface with no dark bar.

- [x] G4: unslop review of the top of the page found nothing left: no stray dark band against the blue, no gradient or glow, no uppercase tracked labels
  EVIDENCE: reviewed against unslop checklist on the 1440px screenshot: header, hero and ruler share one surface; no gradients, glow or colour blobs; labels are sentence case (the medallion ring text was uppercase and is now sentence case); one accent; contrast of every text colour on cobalt checked by G1.
