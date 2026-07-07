# Asha Procgen

Dungeon procgen incubator for richer ASHA generated-level experiments.

This repository is an ASHA downstream project. It should consume `asha-engine`
through public package roots, generated contracts, documented runtime/session
surfaces, and local prototype evidence. It should not import engine internals or
silently patch a sibling engine checkout.

## Fresh Setup

Clone beside `asha-engine`:

```bash
cd /home/dev
git clone git@github.com:FuzzySlipper/asha-engine.git asha-engine
git clone git@github.com:FuzzySlipper/asha-procgen.git asha-procgen
cd asha-procgen
npm install
```

## Verification

```bash
npm run verify
```

Focused checks:

```bash
npm run check:asha-boundary
npm run typecheck
npm run rust:check
npm run rust:test
```

## ASHA Boundary

Use public ASHA package roots and documented subpaths only. If a prototype needs
a missing public ASHA capability, capture a minimal reproduction here and move
the engine-owned work upstream to `asha-engine`.

The local Rust lane in `procgen-rs/` is for downstream preflight tooling,
prototype generation evidence, and project-specific experiments. Generic
runtime/session, collision, pathfinding, render projection, protocol/codegen,
and replay authority belong upstream.
