# Multimodal workshop layout

## Purpose

The workshop is the primary authoring context for `.unibrow` projects. It lets
people assemble timed sequences from mixed assets, preview the result, switch
the same stage to a local live compositor, record, and export.

The workshop is one workspace. Create, Preview, LIVE, and Recording are modes
of the same layout, not separate pages or stacked panels.

## User stories

### Build a sequence

As a creator, I can add images, video, PDF pages, titles, audio, and MIDI from a
Sources rail, then place them on appropriate timeline tracks without leaving
the project.

### Trim and reorder

As an editor, I can drag a clip to change its start time and drag either edge to
trim it. Selecting a clip exposes precise start and duration fields in the
Inspector. Routine timing work does not require editing a table of fields.

### Preview

As a creator, I can scrub one playhead and see the programme result on a stage
that remains large enough to judge framing and typography.

### Go live

As a presenter, I can switch the existing Stage from Preview to LIVE. The
workspace does not move. LIVE is stated in text and restrained semantic colour,
and source health and mixer controls replace clip properties in the Inspector.

### Record

As a presenter, I can start recording while LIVE. Recording is visibly distinct
from both Preview and LIVE through the label `REC`, stronger emphasis, and
control state. The stage and timeline do not jump or resize.

### Export

As a creator, I can open one Export sheet from the Project Command Manager,
choose format and quality, and start export. Export options do not occupy the
editing canvas or appear as a stack of page buttons. Global Save remains in the
title bar and is not conflated with Export.

## Core workflows

### Build from assets

1. Open a `.unibrow` project.
2. Add or import an asset from the Project Command Manager.
3. Select it in Sources.
4. Add it to the appropriate timeline track.
5. Select the new clip and adjust timing in the Inspector if precision is
   needed.

### Trim and reorder

1. Select a clip block.
2. Drag its body to move it along its track.
3. Drag either edge to trim.
4. Use Inspector Start and Duration for exact values.
5. Use Split or Delete clip from the Project Command Manager.

### Preview

1. Choose Preview in the Project Command Manager mode group.
2. Scrub or click the timeline ruler.
3. Observe the result on Stage.
4. Add markers from the Project Command Manager when needed.

### Live and record

1. Choose LIVE.
2. Confirm source health in Inspector.
3. Confirm the Stage header says LIVE.
4. Choose Record.
5. Confirm the Stage header says REC and Record becomes Stop.
6. Stop recording without leaving the workspace.

### Export

1. Choose Export in the Project Command Manager.
2. Select package, preview video, or H.264 in the Export sheet.
3. Set quality and destination.
4. Start export and receive truthful progress/status feedback.

## Layout specification

### Fixed workbench

The document viewport contains one grid:

```text
┌──────────────┬──────────────────────────────┬─────────────────┐
│ Sources      │ Stage                        │ Inspector       │
│              │                              │                 │
├──────────────┴──────────────────────────────┴─────────────────┤
│ Timeline dock: ruler, playhead, tracks, clip blocks           │
└───────────────────────────────────────────────────────────────┘
```

- Desktop grid columns: Sources 220 px, Stage minmax(0, 1fr), Inspector 260 px.
- Timeline dock: 260 px at 1600x1000 and 230 px at 1280x800.
- At narrow desktop widths, Sources and Inspector may reduce to 180 px, but
  remain visible for the evidence sizes.
- Every region participates in normal grid flow. No absolute-positioned region
  may overlap another region.
- Absolute positioning is limited to semantic overlays inside bounded regions:
  clip blocks and playhead inside timeline tracks, and programme content inside
  the Stage frame.

### Sources

- Joined left rail, not a card.
- One source per row: type badge, readable name, concise asset/type metadata.
- Row actions appear on selection or in a quiet menu. They must not be clipped.
- Add asset is in the Project Command Manager; the rail may accept file drop.
- Missing sources show an explicit Missing badge and also appear in Problems.

### Stage

- Fills available centre space while preserving project aspect ratio.
- Default programme ratio is 16:9.
- Uses a recessed surround and one centred programme frame.
- In Create/Preview it renders the composition at the playhead.
- In LIVE it renders the local compositor in the same frame.
- A compact header inside the Stage states Preview, LIVE, or REC. Red is used
  only for LIVE, REC, or errors. Text always carries the state.
- No empty bordered placeholder is shown.

### Inspector

- Joined right rail with a stable heading.
- Source selected: source metadata and health.
- Clip selected: label, track, start, duration, and type-specific properties.
- LIVE/REC: source health and mixer controls.
- Numeric timing fields exist only here, never repeated in timeline rows.
- Inspector scrolls internally and never disappears behind Stage or header.

### Timeline dock

- One ruler and one playhead span all tracks.
- Track rows:
  - Video / Image
  - Title
  - Audio / MIDI
- Clips are blocks positioned by start time and sized by duration.
- Clip label and duration remain readable when space permits.
- Body drag moves a clip. Left/right handles trim.
- Selection is visible by outline and is not colour-only.
- Snap, markers, split, delete, and timeline zoom live in the Project Command
  Manager.
- The timeline may scroll horizontally; it must not become a form or list of
  clip property rows.

### Project Command Manager family

The Project family replaces Browse for `.unibrow`:

- Add: Add asset, Add title.
- Edit: Split, Delete clip.
- Timeline: Snap, Marker, Zoom timeline.
- Mode: Preview, LIVE, Record/Stop.
- Output: Export.

Unavailable commands remain in place with a reason. Save remains the one global
primary action in the title bar.

### Export sheet

- Opened by Project CM Export.
- Non-modal sheet or docked panel where possible.
- Format: project package, share package, preview MP4, H.264 MP4.
- Quality/preset and destination follow format.
- The sheet does not obscure the Stage state and can be dismissed without
  losing edits.

### Responsive behaviour

- At 1600x1000, all four regions are simultaneously visible.
- At 1280x800, Sources and Inspector narrow and the timeline becomes slightly
  shorter; Stage remains 16:9 and usable.
- Below the specified evidence width, one side rail may become a switchable
  dock, but the timeline remains in flow.

## Acceptance and evidence

- Correct Project Command Manager family.
- No overlap or clipping at 1600x1000 or 1280x800.
- Stage content is readable in light and dark themes.
- Source names and selected clip text are readable.
- Timeline is track-based with blocks proportional to duration.
- Preview, LIVE, and Recording keep identical geometry.
- Capture matrix: Create, LIVE, Recording at both dimensions in light and dark.
- Any overlap, clipping, unreadable text, or wrong Command Manager family is an
  automatic Fail.
