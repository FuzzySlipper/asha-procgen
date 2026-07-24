# Piece Assembly Contract

Status: first contract for catalog-driven 2D piece assembly.

This layer sits after `geometry_2d` and before any mesh, voxel, renderer, or
runtime output. It turns abstract room/corridor geometry into explicit
placeable pieces chosen from a reusable shape catalog. Rooms, corridors, bends,
thresholds, landings, and later vertical connectors are all represented as
pieces with exits and footprints.

## Pipeline

```text
candidate + intermediate_breakdown + geometry_2d
  -> piece_build_plan
  -> shape_catalog matching
  -> piece_shape_match
  -> piece_placement
  -> piece placement validation
  -> viewer build evidence
  -> future mesh/voxel output
```

The graph and intermediate layers answer "what should this dungeon do?" The
piece layer answers "which explicit cells and exits does this build require?"
The mesh/voxel layer, when it exists, will answer "how is this rendered or
simulated?"

## Shape Catalog Artifact

Kind: `asha_procgen.shape_catalog.v1`

Shape catalogs are reusable prefab metadata. A shape may eventually point at
art assets or voxel volumes, but this first contract is JSON-only and 2D-grid
focused.

Important top-level fields:

- `catalogId`: stable catalog id.
- `schemaVersion`: additive schema version.
- `cellSize`: authoring grid size in abstract cells.
- `placementPolicy`: versioned room-boundary, clearance, wall, and doorway
  policy copied into each placement artifact.
- `shapes`: reusable shape records.

Placement policy schema v1 has one supported contact mode,
`glued_exits_only`, and requires `preservePieceBoundaries: true`. Its tunable
values are `minimumClearanceCells` and `wallThicknessCells`.
`doorwayWidthCells` is versioned policy data, but schema v1 deliberately
accepts only `1`; wider openings fail closed until placement routing owns their
complete oriented footprint. Clearance must be at least
`2 * wallThicknessCells + 1`; this leaves a route cell between the wall
envelopes of separate pieces. Hinted origins are deterministically expanded by
clearance plus wall thickness before the local placement search, so a dense
geometry hint does not silently collapse room boundaries.

Important shape fields:

- `shapeId`: stable shape id such as `shape.room.standard.2_exit`.
- `pieceKinds`: compatible requirement kinds, such as `room`, `corridor`,
  `bend`, `threshold`, `reward`, `hazard`, `boss`, `secret`, or `resource`.
- `footprint`: occupied grid cells relative to shape origin.
- `reservedCells`: optional clearance cells that must remain empty but are not
  occupied.
- `exits`: exit sockets with relative position, direction, width, and tags.
- `allowedTransforms`: transforms the assembler may use, such as `identity`,
  `rotate90`, `rotate180`, `rotate270`, `mirrorX`, or `mirrorY`.
- `featureSockets`: gameplay/art sockets such as `container`, `boss_space`,
  `gate_line`, `hazard_zone`, `reward_cache`, `key_pickup`, `secret_marker`,
  `shortcut_marker`, or `resource_clue`.
- `tags`: matching hints such as `small`, `wide`, `locked_threshold`,
  `pressure`, `hidden`, `shortcut`, `rejoin`, or `landmark`.

### Catalog Exit Model

Exit directions use a 2D vocabulary now:

- `north`
- `east`
- `south`
- `west`

Later 3D catalogs may add `up`, `down`, or vector-style exits. The 2D matcher
must reject unsupported 3D exits until placement validation can prove them.

Two catalog-piece exits can glue when the build plan links their pieces, their
mapped directions are opposite, their widths are compatible, and required tags
are satisfied. A procedural room-to-room link instead preserves each room
exit's authored outward direction and the geometry lane joining them; the two
room exits need not face opposite directions when the lane bends. Placement
owns the physical route between separated footprints in either mode, and exit
compatibility does not grant arbitrary cell contact.

Placement is a bounded deterministic search rather than a one-pass atlas
layout. Room-shaped requirements are placed first, room-facing connector
pieces are anchored from their matched exit coordinates, and interior
corridor/bend pieces retain the source geometry lane hints. Connection routes
then try four documented orderings without relaxing occupancy, wall,
clearance, section-crossing, or shared-room approach checks. If the compact
realization cannot route, one expanded scale tier is tried before the candidate
is rejected. Batch realization additionally tries four deterministic
shape/transform alternatives, changing one requirement at a time and recording
the selected candidate rank/count instead of silently replacing the match.

Every successful `piece_placement` records `realizationSearch`, including the
selected scale tier, realization attempts, route-order attempt, and route
attempt count. Repeating the same candidate, catalog, match report, and seed
must reproduce this evidence and the complete placement byte-for-byte.

## Piece Build Plan Artifact

Kind: `asha_procgen.piece_build_plan.v1`

The build plan is a requirement graph, not yet a placed map. It expands the
geometry artifact into explicit pieces and connectors for catalog matching.

Important fields:

- `planId`: stable generated id.
- `candidateId`: source candidate id.
- `geometryId`: source geometry id.
- `corridorRealization`: exactly `catalog` or `procedural`.
- `sourceCandidateRef`: source candidate ref.
- `sourceGeometryRef`: source `geometry_2d` ref.
- `sourceIntermediateRef`: source intermediate breakdown ref.
- `requirements`: piece requirements.
- `links`: exit-to-exit requirements between pieces.
- `contentRequirements`: content/socket requirements that must be satisfied by
  matched shapes or placed feature sockets.

Important requirement fields:

- `pieceId`: stable requirement id.
- `kind`: `room`, `corridor`, `bend`, `threshold`, `reward`, `hazard`, `boss`,
  `secret`, `shortcut`, `resource`, or other additive piece kind.
- `role`: semantic role inherited from geometry/intermediate.
- `sourceRefs`: graph node/edge, intermediate region/connector, and
  geometry room/corridor refs.
- `requiredExits`: abstract exits the selected shape must provide.
- `requiredSockets`: feature sockets needed for contents or gameplay beats.
- `tags`: matching hints and validation semantics.
- `placementHints`: optional non-authoritative hints, such as preferred
  approximate cell span or corridor length band.

In `catalog` mode, corridors are first-class shape requirements. A geometry
corridor may expand into:

```text
connector piece -> straight corridor piece -> bend piece -> straight corridor piece -> connector piece
```

or into a shorter equivalent.

In `procedural` mode, room and feature requirements remain catalog matched, but
connector/corridor/bend requirements are omitted. Each geometry corridor emits
one direct room-to-room link carrying the complete planned polyline. Placement
routes cells only inside a deterministic bounded envelope around that lane and
continues to enforce room clearance, exclusive physical-section ownership,
portal provenance, and built-flow equivalence. It may adjust within the
envelope to reach transformed room exits; it may not invent an unrelated
shortest path. Unknown modes reject during typed deserialization.

The mode applies to the whole build. Automatic per-corridor mixing is not part
of this contract.

## Piece Shape Match Artifact

Kind: `asha_procgen.piece_shape_match.v1`

Shape match reports select catalog shapes for each build-plan requirement
without placing them on an occupancy grid yet.

Important fields:

- `matchId`: stable generated id.
- `planId`: source build-plan id.
- `catalogId`: source shape catalog id.
- `seed`: deterministic tie-break seed.
- `matches`: selected piece/shape/transform records.
- `rejections`: agent-readable rejected shape reasons.
- `diagnostics`: fatal diagnostics for unmatched requirements.

Important match fields:

- `pieceId`: source requirement id.
- `requirementKind`: source piece kind.
- `shapeId`: selected catalog shape.
- `transform`: selected transform such as `identity` or `rotate90`.
- `exitMap`: requirement exits mapped to transformed catalog exits.
- `socketMap`: required feature sockets mapped to catalog sockets.

Matching filters by piece kind, required sockets, exit count, exit direction,
and width. It considers allowed rotations/reflections and uses deterministic
seeded tie-breaking when multiple shapes score equally. Rejections explain
kind mismatches, missing sockets, exit-count gaps, and transform-specific exit
compatibility failures. Each matched exit retains its transformed shape-local
`x`/`y`, direction, and width; placed instances translate those coordinates to
absolute placement cells.

## Piece Placement Artifact

Kind: `asha_procgen.piece_placement.v1`

Placement artifacts record selected catalog shapes and concrete grid cells.
They are the first layer that owns occupancy.

Important fields:

- `placementId`: stable generated id.
- `planId`: source build-plan id.
- `catalogId`: source shape catalog id.
- `matchId`: source shape match id.
- `sourcePlanRef`: source build-plan ref.
- `sourceCatalogRef`: source shape catalog ref.
- `sourceMatchRef`: source shape match ref.
- `cellSize`: placement grid cell size.
- `gridConnectivity`: physical grid adjacency policy, currently `four_way`
  or `eight_way`; CLI assembly defaults to `four_way`.
- `placementPolicy`: the exact catalog policy used to place and later extrude
  the build.
- `instances`: placed piece instances.
- `gluedExits`: validated exit-to-exit joins.
- `occupiedCells`: optional flattened occupancy index for quick inspection.
- `connectionCells`: generated bridge cells that make glued joins physically
  reachable under `gridConnectivity`.
- `reservedCells`: optional flattened reservation/clearance index.
- `danglingExits`: exits intentionally left open or invalid exits found during
  validation.

Occupied cells from different build pieces must remain beyond the configured
minimum clearance even when the pieces are linked. A glued exit is represented
only by owned `connectionCells`; it is not blanket permission for two
footprints to touch. The route search rejects occupied/reserved crossings and
keeps every route outside the wall envelope of unrelated pieces. If no origin
or route satisfies those constraints, assembly fails instead of falling back
to an unsafe straight bridge.

Each glued exit carries both placed endpoint cells, directions, and widths.
Its owned connection route must include both endpoint cells, remain connected,
and may enter an endpoint piece's wall envelope only through the outward tunnel
defined by that exact transformed exit. Non-exit boundary cells are therefore
walls even when another boundary happens to be closer. Placement protects an
outward approach lane for every matched exit so later pieces cannot box in a
declared route.

Important instance fields:

- `instanceId`: stable instance id.
- `pieceId`: source requirement id.
- `shapeId`: selected catalog shape.
- `transform`: selected transform.
- `origin`: grid origin.
- `occupiedCells`: transformed occupied cells.
- `reservedCells`: transformed reserved cells.
- `exitMap`: mapping from requirement exits to transformed catalog exits.
- `featurePlacements`: mapping from content/socket requirements to feature
  sockets.

## Validation Artifact

Kind: `asha_procgen.validation.piece_placement.v1`

Placement validation should be deterministic and fail closed. Expected
diagnostic families:

- catalog shape missing or unsupported;
- required exit unsatisfied;
- incompatible glued exits;
- occupied-cell overlap;
- configured minimum-clearance violations between piece instances;
- reserved-cell conflict;
- undeclared, occupied, reserved, or wall-clearance-violating connection cells;
- dangling required exit;
- missing feature socket;
- start-to-goal unreachable through glued exits;
- piece instance unreachable on the physical placement grid;
- unsupported vertical/3D exit.

Fatal diagnostics block sample evidence and future mesh/voxel output. Warnings
may flag density, excessive corridor length, or stylistic mismatch.

## Responsibilities By Layer

Graph/intermediate:

- progression, lock/key order, branch purpose, pressure/reward semantics;
- no shape ids, grid cells, mesh handles, or voxel coordinates.

Geometry:

- approximate 2D room rectangles, corridor polylines, content labels;
- enough coordinate intent to seed piece requirements;
- no final occupancy authority.

Piece build plan:

- explicit room/corridor/bend/threshold requirements;
- source refs and required exits/sockets;
- no selected catalog shape or occupied cells.

Piece shape match:

- selected catalog shapes, transforms, exit maps, and socket maps;
- rejected alternatives and unmatched requirement diagnostics;
- no occupied cells or glued-exit authority yet.

Piece placement:

- selected shapes, transforms, occupied cells, reserved cells, glued exits;
- validation authority for prefab-grid assembly;
- still no mesh, voxel, renderer, collision, or ASHA runtime authority.

## Fixture Catalog Shape Families

The first fixture catalog should include at least:

- standard chambers with 1, 2, 3, and 4 exits;
- straight corridor pieces;
- bend corridor pieces;
- connector/landing pieces;
- locked threshold or interior gate split pieces;
- reward/key/cache pockets;
- hazard/pressure rooms;
- boss/preparation rooms;
- secret/shortcut marker pieces;
- resource/clue rooms.

The first fixture catalog lives at:

```text
fixtures/shape-catalogs/2d-basic.json
```

It is intentionally metadata-only and small enough for agents to inspect by
hand while still covering the current sample dungeon vocabulary.

## Non-Goals

- No final tile-perfect roguelike map.
- No mesh generation.
- No voxel output.
- No ASHA runtime, renderer, collision, or pathfinding integration.
- No hand-authored art assets beyond JSON fixture shape metadata.
- No accepted 3D placement yet; 3D exits remain future contract vocabulary
  until validators can prove them.
