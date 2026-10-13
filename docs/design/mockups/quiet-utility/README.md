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
| `chrome-before-after.html` | `chrome-before-after.png` | 1280x800 | Annotated 176 px to 80 px chrome comparison |
| `text-toolbar-compact-chrome.html` | `text-toolbar-compact-chrome-markdown.png`, `text-toolbar-compact-chrome-docx.png` | 1280x112 | Approved title + 32 px text compact row + CM chrome |
| `command-system-gallery.html` | `command-system-gallery-light.png`, `command-system-gallery-dark.png` | 1920x1080 | Control states, stable text family, and mod toolbar |

Command Manager density studies:

| Context | Compact | Large |
| --- | --- | --- |
| Text document | `document-command-compact.png` | `document-command-large.png` |
| PDF annotation | `pdf-command-compact.png` | `pdf-command-large.png` |
| CAD/3D wide | `cad-command-compact-wide.png` | `cad-command-large-wide.png` |

Run `./render.sh` to regenerate all PNGs with an isolated Chromium profile.

## Accepted chrome

- 40 px unified custom title bar: menu, tabs, location/command field, global
  actions, drag region, platform window controls.
- 40 px Compact Command Manager (default): named groups with 16 px Lucide icon
  + short label controls, split buttons, deterministic group overflow, and
  Follow/Pin.
- 64 px Large Command Manager: 20 px icons above labels, for a 104 px total
  chrome budget.
- 80 px Compact budget; 104 px Large; 40 px when the manager is collapsed.
- 40 px collapsed activity rail with Lucide icons. Expanded, auto-hide, and
  hidden are preferences. The rail opens panels only.
- No Browse/Text/Create/Live switcher. Saved layouts live in View > Workspaces
  and the palette.

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

1. The synthetic CAD viewport proves hierarchy and density, not rendering
   quality or actual DXF interaction.
2. The light document AI transcript is intentionally sparse. Real conversation
   content will need careful density testing so it does not become a bubble
   feed or compete with the document.
3. A 40 px title bar is sufficient at 100% scaling, but 150-200% Windows
   scaling and Linux client-side decoration metrics remain implementation
   acceptance tests.

The concept references are direction only. These mockups deliberately remove
their large rounded chat bubbles, glossy toggles, and decorative softness.
