# FontCraft site: type system

Four families, each with one job. Nothing else is used.

| Role | Family | Used for | Notes |
|---|---|---|---|
| Display | Fraunces Variable | h1, h2, h3, the Aa readout in the ruler | Weight 600. Axes: opsz 100, SOFT 30, WONK 1 for headlines; opsz 60, SOFT 40 for subheads. Tracking tight (-0.055em display, -0.03em h2, -0.02em h3). |
| Text | Archivo Variable | Paragraphs, buttons, table text | Weights 400 and 500. Line height 1.6. |
| Mono | IBM Plex Mono | Labels, section bars, numbers, file names, captions | 10-11px, tracking 0.02em, sentence case (no uppercase). |
| Script | Noto Naskh Arabic | Arabic samples only | Never for interface text. |

## Scale

| Token | Size | Where |
|---|---|---|
| --fs-display | 3.6rem to 8.8rem (fluid) | Hero headline only |
| --fs-h2 | 2.4rem to 4.6rem | Section headings |
| --fs-h3 | 1.45rem to 2rem | Highlight titles |
| --fs-lede | 1.05rem to 1.28rem | Intro paragraphs |
| --fs-body | 1rem | Body |
| --fs-small | 0.875rem | Table cells, notes |
| --fs-label | 11px | Labels and meta (mono) |

Rules: one display size per page, hierarchy from size and colour rather than weight, sentence case everywhere,
no italic, numbers in mono. Change values in `src/styles/global.css` (`:root`); components read the tokens.
