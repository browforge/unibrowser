# Quiet Utility mockups

Static HTML/CSS studies for the normative
[visual language](../../visual-language.md). Render with Chrome at the dimensions
in each filename.

| Source | PNG | Size | Purpose |
| --- | --- | --- | --- |
| `document-problems-ai.html` | `document-problems-ai.png` | 1280x800 | Text/code, Problems, docked AI |
| `document-problems-ai-dark.html` | `document-problems-ai-dark.png` | 1280x800 | Dark version of the same workspace |
| `pdf-annotation.html` | `pdf-annotation.png` | 1280x800 | PDF pages, canvas, annotation properties |
| `unibrow-create.html` | `unibrow-create.png` | 1280x800 | Create workspace and video timeline |
| `new-tab.html` | `new-tab.png` | 1280x800 | Recents and active downloads |
| `multimodal-workshop.html` | `multimodal-workshop.png` | 1280x800 | Dense CAD, code, timeline, Problems, inspector |
| `multimodal-workshop-wide.html` | `multimodal-workshop-wide.png` | 1920x1080 | Wide multimodal layout with Layers and scopes |

## Drift review

The set passes the applicable drift checklist:

- one joined workbench rather than a card grid;
- 0, 4, and 6 px structural radii, with one 12 px permission-state pill;
- shadows only on the physical PDF page;
- dark slate shell, warm task surfaces, and one blue accent;
- semantic green/amber/red with text or positional redundancy;
- all functional text at least 12 px;
- no glossy controls, chat bubbles, glass, promotional illustration, or
  engagement UI;
- enough work remains visible at 1280x800.

Weakest areas:

1. The rail labels are correct in placement and density but the HTML studies do
   not yet include the final Lucide glyph for every destination.
2. The synthetic CAD viewport proves hierarchy and density, not rendering
   quality or actual DXF interaction.
3. The light document AI transcript is intentionally sparse. Real conversation
   content will need careful density testing so it does not become a bubble
   feed or compete with the document.

The concept references are direction only. These mockups deliberately remove
their large rounded chat bubbles, glossy toggles, and decorative softness.
