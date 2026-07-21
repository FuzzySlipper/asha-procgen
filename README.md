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
npm run publish:asha-smoke
npm run viewer:smoke
```

## ASHA Prefab and ProjectBundle Publishing Proof

The downstream publishing adapter maps representative Procgen shape matches
and placements to generated ASHA prefab identities and a ProjectBundle-shaped
durable artifact inventory:

```bash
npm run publish:asha-smoke
```

It uses public `@asha/contracts` and `@asha/game-workspace` roots, preserves
generation provenance, and fails closed on missing mappings/roles, incompatible
assets, duplicate identities, and invalid transforms. See
[`docs/asha-publishing-boundary.md`](docs/asha-publishing-boundary.md) for the
ownership, distribution, non-claims, and crate-disposition contract.

## Native ASHA Voxel Extrusion Proof

The separate engine-backed authority smoke extrudes a validated 2D piece placement into
a simple enclosed voxel volume. Placement `x/y` maps to ASHA voxel `x/z`; the
proof adds a floor, three-voxel walls, and a ceiling, then submits bounded
`generateChunk`, `fillRegion`, and `setVoxel` batches through a Rust-backed
public `RuntimeSession`.

The source placement carries a versioned policy for minimum inter-piece
clearance, wall thickness, and doorway width. Occupied cells retain their piece
owners through extrusion; walls surround the separated footprints and only
declared glued-exit connection routes become openings. The compiler rejects
unsafe policy combinations and routes that would open an unrelated piece.

```bash
npm run voxel:asha-smoke
```

The command regenerates
`artifacts/evidence/native-voxel-extrusion.json` with deterministic authority
voxel-state hashes, command-phase receipts, and bounded comparison readbacks.

The smoke test requires the sibling `asha-engine` checkout and its built native
addon at `ts/packages/native-bridge/dist/native-bridge.node`. It proves native
command acceptance, deterministic authority voxel-state hashes, and fail-closed
unknown-material rejection. A separate voxel-conversion comparison preserves
bounded model/material readback coverage, but it is not the mutation path under
test. The proof does not claim 3D piece placement, exit-socket alignment,
rendering, navigation, or performance evidence.

## Engine Voxel Inspection

The LAN viewer keeps the existing isometric `Voxel` evidence tab and adds a
separate `Voxel 3D` inspection tab. The 3D view compiles the same placement
extrusion, omits only its ceiling from the presentation frame, and mounts the
public `@asha/renderer-host` inspection surface with its procedural grid, mouse
or arrow-key orbit, focused W/A/S/D movement, and keyboard/wheel zoom. It is
projection-only visual evidence, not RuntimeSession,
collision, navigation, native-render, or performance authority.

## ASHA Boundary

Use public ASHA package roots and documented subpaths only. If a prototype needs
a missing public ASHA capability, capture a minimal reproduction here and move
the engine-owned work upstream to `asha-engine`.

The local Rust lane in `procgen-rs/` is for downstream preflight tooling,
prototype generation evidence, and project-specific experiments. Generic
runtime/session, collision, pathfinding, render projection, protocol/codegen,
and replay authority belong upstream.
