# Proving mod: `base-dxf`

**Status:** Blitz placeholder · not a production CAD kernel.

## What ships

- Opens `.dxf` via bundled mod `base-dxf` (`VIEWER_KIND: dxf`).
- Desktop viewer: layer list, **geometry entity summary**, and a **wireframe** preview (Three.js line segments; falls back to a simple SVG proxy when WebGL is unavailable).
- Command palette lists **Measure** and **Export STEP**; they stay **greyed out** with reason *Proving mod: requires full CAD engine*.

## What does not ship

- NURBS/solid modelling, constraints, or accurate unit metadata.
- STEP/IGES export, snapping, or dimension styles.

## Intent

Prove the mod boundary (handler + viewer + disabled CAD commands) before investing in a real engine or community `.ubmod`.

See [ADR 0002](adr/0002-mod-system-and-sandboxing.md) and the main mod portal for installing extended handlers.
