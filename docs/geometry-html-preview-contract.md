# Geometry And HTML Preview Contract

Status: first contract for generated 2D dungeon floor-plan previews.

This contract starts the layer after `intermediate_breakdown`: concrete enough
to draw rooms and corridors, still separate from tile maps, game runtime state,
meshes, voxels, or renderer integration.

## Pipeline

```text
candidate + intermediate breakdown
  -> geometry_2d layout
  -> geometry validation
  -> html_preview metadata + standalone HTML/SVG file
  -> optional smoke screenshot evidence
```

## Geometry Artifact

Kind: `asha_procgen.geometry_2d.v1`

Initial command:

```bash
npm run procgen -- geometry emit-2d \
  --candidate <candidate.json> \
  --intermediate <intermediate-breakdown.json> \
  --seed <u64> \
  --out <geometry.json>
```

Important fields:

- `geometryId`: stable generated id.
- `candidateId`: source candidate id.
- `seed`: deterministic layout seed.
- `sourceCandidateRef`: source candidate path.
- `sourceIntermediateRef`: source intermediate breakdown path.
- `bounds`: total drawing bounds and grid size.
- `rooms`: variable room rectangles with role, footprint class, geometry role,
  source region, and source node refs.
- `corridors`: routed corridor polylines with width, traversal hint, source
  connector, source edge, and semantic tags.
- `contents`: lightweight room annotations for preview labels. Each annotation
  has an `id`, `roomId`, `sourceRef`, `kind`, `label`, and style/function tags.
  Current kinds include `start_marker`, `goal_marker`, `key_pickup`,
  `locked_gate`, `boss_threshold`, `reward_cache`, `hazard`,
  `resource_clue`, `shortcut_marker`, and `secret_route_marker`.
- `skippedConnectors`: explicit skipped connector records when the emitter
  cannot or should not draw a connector.

The geometry artifact is allowed to use coordinates and rectangles. It is not a
tile map and does not imply collision, runtime navigation, mesh, voxel, or ASHA
renderer authority.

The first `emit-2d` implementation places variable room footprints from
intermediate regions, routes simple semantic corridor polylines, and derives
preview content annotations from graph/intermediate semantics.

## Geometry Validation

Kind: `asha_procgen.validation.geometry_2d.v1`

Command:

```bash
npm run procgen -- geometry validate-2d \
  --state <geometry.json> \
  --out <geometry.validation.json>
```

The validator checks valid room bounds, non-overlapping room rectangles,
corridor room refs, corridor endpoint attachment, connector represented/skipped
bookkeeping inside the artifact, directed start-to-goal reachability, semantic
refs for locked/hidden/shortcut corridors, and content room anchors.

## HTML Preview Artifact

Kind: `asha_procgen.html_preview.v1`

Command:

```bash
npm run procgen -- preview html \
  --geometry <geometry.json> \
  --validation <geometry.validation.json> \
  --out <preview.html>
```

By default the command refuses invalid geometry. Pass `--allow-invalid` to render
a diagnostic preview for debugging.

This metadata records the relationship between:

- `geometryRef`;
- `validationRef`;
- `htmlRef`;
- optional `screenshotHint`.

The HTML file itself should be standalone: dark CSS, inline SVG, labels,
legend/metadata, and no dev server requirement.

## Compatibility

Existing `asha_procgen.layout_2d.v1` remains the simple graph embedding used by
the preview site. It is not replaced by `geometry_2d`.

`asha_procgen.intermediate_breakdown.v1` remains the semantic pre-geometry
handoff. `geometry_2d` consumes it and may fail validation if the geometry
artifact loses required source refs, connectors, or content anchors.

## Non-Goals

- No tile-perfect roguelike map.
- No final renderer or ASHA runtime integration.
- No mesh generation.
- No voxel output.
- No hand-authored art assets.
