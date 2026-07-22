# Agent Runbook

Status: v2 sample runbook for the CLI workbench and viewer.

## Install

```bash
npm install
```

## Build A Sample Run

```bash
npm run baseline
```

This writes a deterministic example under:

```text
artifacts/samples/first-run/
```

Important files:

- `candidate-000-base.json`
- `candidate-001-lock_key_loop.json`
- `candidate-002-optional_treasure_detour.json`
- `candidate-003-one_way_shortcut.json`
- `candidate-004-secret_bypass.json`
- `validation.graph.json`
- `score.graph.json`
- `layout-2d.json`
- `accepted.json`
- `transcript.jsonl`

## Build A Batch Run

```bash
npm run batch:sample
```

This writes a deterministic v2 batch under:

```text
artifacts/samples/batch-v2/
```

Important files:

- `selection-report.json`
- `candidate-000/accepted.json`
- `candidate-000/validation.graph.json`
- `candidate-000/analysis.graph.json`
- `candidate-000/compatible-rules.json`
- `candidate-000/spatial-intent.json`
- `candidate-000/intermediate-breakdown.json`
- `candidate-000/intermediate.validation.json`
- `candidate-000/score.graph.json`
- `candidate-000/transcript.jsonl`

The sample command generates 10 candidates from:

```text
fixtures/batch-profiles/v2-sample.json
```

The selection report records the profile id/ref, the profile sequence used for
each candidate, topology fingerprints, budget checks, and sorts accepted entries
by deterministic selection score. Accepted entries also carry refs to graph
analysis, compatible rules, spatial intent, intermediate breakdown, and
intermediate validation artifacts.

`npm run batch:sample` also emits the full generated dungeon preview stack for
each accepted candidate:

```text
artifacts/samples/batch-v2/<candidate>/geometry-2d.json
artifacts/samples/batch-v2/<candidate>/geometry-2d.validation.json
artifacts/samples/batch-v2/<candidate>/geometry-2d.preview.html
artifacts/samples/batch-v2/<candidate>/html-preview.json
artifacts/samples/batch-v2/<candidate>/shape-catalog.report.json
artifacts/samples/batch-v2/<candidate>/piece-plan.json
artifacts/samples/batch-v2/<candidate>/piece-shape-match.json
artifacts/samples/batch-v2/<candidate>/piece-placement.json
artifacts/samples/batch-v2/<candidate>/piece-placement.validation.json
```

Each `accepted` entry in `selection-report.json` carries `geometryRef`,
`geometryValidationRef`, `htmlPreviewRef`, `htmlRef`, `shapeCatalogRef`,
`catalogInspectionRef`, `piecePlanRef`, `shapeMatchRef`, `piecePlacementRef`,
and `piecePlacementValidationRef`. The viewer Build tab prefers the catalog
piece placement grid and falls back to the older geometry-rasterized grid when
piece artifacts are absent.

## Manual CLI Sequence

```bash
npm run procgen -- init \
  --intent fixtures/intents/first-slice.intent.json \
  --seed 4103 \
  --out artifacts/manual/candidate-000-base.json \
  --receipt artifacts/manual/receipt-000-init.json \
  --transcript artifacts/manual/transcript.jsonl

npm run procgen -- graph apply-rule \
  --state artifacts/manual/candidate-000-base.json \
  --rule lock_key_loop \
  --seed 4104 \
  --out artifacts/manual/candidate-001-lock_key_loop.json \
  --receipt artifacts/manual/receipt-001-lock_key_loop.json \
  --transcript artifacts/manual/transcript.jsonl

npm run procgen -- validate graph \
  --state artifacts/manual/candidate-001-lock_key_loop.json \
  --out artifacts/manual/validation.graph.json

npm run procgen -- score graph \
  --state artifacts/manual/candidate-001-lock_key_loop.json \
  --out artifacts/manual/score.graph.json
```

Use `npm run procgen -- graph summarize --state <candidate>` to print a compact
agent-readable graph summary.

Fork before trying alternate plans:

```bash
npm run procgen -- graph fork \
  --state artifacts/manual/candidate-001-lock_key_loop.json \
  --label boss-prep-attempt \
  --seed 4201 \
  --out artifacts/manual/candidate-001a-boss-prep-fork.json \
  --receipt artifacts/manual/receipt-001a-fork.json \
  --transcript artifacts/manual/transcript.jsonl
```

For machine-readable planning context:

```bash
npm run procgen -- graph rules --out artifacts/manual/rules.json

npm run procgen -- graph summarize \
  --state artifacts/samples/batch-v2/candidate-005/candidate-007-branch_merge_shortcut.json \
  --json \
  --out artifacts/manual/summary.json

npm run procgen -- analyze graph \
  --state artifacts/samples/batch-v2/candidate-005/candidate-007-branch_merge_shortcut.json \
  --out artifacts/manual/analysis.json

npm run procgen -- graph compatible-rules \
  --state artifacts/samples/batch-v2/candidate-005/candidate-007-branch_merge_shortcut.json \
  --out artifacts/manual/compatible-rules.json
```

Implemented richer graph rules:

```text
hub_spoke_cluster
nested_lock_key_chain
hazard_resource_tradeoff
boss_preparation_loop
gated_treasure_branch
branch_merge_shortcut
```

Duplicate or incompatible rule applications are rejected with receipt
diagnostics and `repairHint` text where the tool can suggest a next edit.

## Intermediate Layout Intent

The pre-geometry graph analysis and breakdown contract is documented in:

```text
docs/intermediate-layout-contract.md
```

A typical manual chain:

```bash
npm run procgen -- annotate spatial-intent \
  --state artifacts/samples/batch-v2/candidate-005/candidate-007-branch_merge_shortcut.json \
  --analysis artifacts/manual/analysis.json \
  --out artifacts/manual/spatial-intent.json

npm run procgen -- breakdown emit \
  --state artifacts/samples/batch-v2/candidate-005/candidate-007-branch_merge_shortcut.json \
  --annotations artifacts/manual/spatial-intent.json \
  --out artifacts/manual/intermediate-breakdown.json

npm run procgen -- breakdown validate \
  --state artifacts/manual/intermediate-breakdown.json \
  --out artifacts/manual/intermediate.validation.json
```

This layer names regions, connectors, and constraints for later geometry passes.
It does not emit rooms, meshes, voxels, or 3D placement.

## Geometry HTML Preview

The generated 2D dungeon preview target is documented in:

```text
docs/geometry-html-preview-contract.md
```

This is the planned path from intermediate breakdowns to standalone HTML/SVG
floor-plan previews with variable rooms, corridors, labels, and contents. It is
separate from the existing simple `layout-2d.json` graph embedding.

Plan physical connections, then emit geometry from that exact plan:

```bash
npm run procgen -- geometry plan-connections \
  --candidate artifacts/samples/batch-v2/candidate-005/candidate-007-branch_merge_shortcut.json \
  --intermediate artifacts/samples/batch-v2/candidate-005/intermediate-breakdown.json \
  --out artifacts/manual/physical-connection-plan.json

npm run procgen -- geometry emit-2d \
  --candidate artifacts/samples/batch-v2/candidate-005/candidate-007-branch_merge_shortcut.json \
  --intermediate artifacts/samples/batch-v2/candidate-005/intermediate-breakdown.json \
  --connection-plan artifacts/manual/physical-connection-plan.json \
  --seed 6101 \
  --out artifacts/manual/geometry-2d.json
```

Validate the emitted geometry before using it as preview evidence:

```bash
npm run procgen -- geometry validate-2d \
  --state artifacts/manual/geometry-2d.json \
  --out artifacts/manual/geometry-2d.validation.json
```

Render the standalone HTML/SVG preview:

```bash
npm run procgen -- preview html \
  --geometry artifacts/manual/geometry-2d.json \
  --validation artifacts/manual/geometry-2d.validation.json \
  --out artifacts/manual/geometry-2d.preview.html
```

## Piece Assembly Preview

The catalog-driven piece assembly target is documented in:

```text
docs/piece-assembly-contract.md
docs/build-piece-library-structure.md
```

This is the path from geometry rectangles/corridors to prefab or voxel-ready
build data. It treats rooms, corridors, bends, thresholds, landings,
reward pockets, hazards, boss spaces, shortcuts, secrets, and resource rooms as
explicit pieces with exits, feature sockets, catalog matches, transformed
occupancy cells, reservations, and glued-exit validation.

Current piece assembly commands:

```bash
npm run procgen -- build catalog inspect \
  --catalog fixtures/shape-catalogs/2d-basic.json \
  --out artifacts/manual/shape-catalog.report.json

npm run procgen -- build emit-piece-plan \
  --candidate artifacts/samples/batch-v2/candidate-005/candidate-007-branch_merge_shortcut.json \
  --geometry artifacts/manual/geometry-2d.json \
  --intermediate artifacts/manual/intermediate-breakdown.json \
  --out artifacts/manual/piece-plan.json

npm run procgen -- build match-shapes \
  --catalog fixtures/shape-catalogs/2d-basic.json \
  --piece-plan artifacts/manual/piece-plan.json \
  --seed 7101 \
  --out artifacts/manual/piece-shape-match.json

npm run procgen -- build assemble \
  --catalog fixtures/shape-catalogs/2d-basic.json \
  --piece-plan artifacts/manual/piece-plan.json \
  --shape-match artifacts/manual/piece-shape-match.json \
  --connectivity four-way \
  --out artifacts/manual/piece-placement.json

npm run procgen -- build validate-placement \
  --state artifacts/manual/piece-placement.json \
  --out artifacts/manual/piece-placement.validation.json
```

Focused smoke:

```bash
npm run piece:smoke
```

Do not treat the current viewer Build tab's geometry-rasterized cells as final
piece-placement authority. The `piece-plan.json` artifact is the requirement
graph, and `piece-shape-match.json` records selected catalog shape ids,
transforms, exit maps, socket maps, and rejected alternatives. The
`piece-placement.json` artifact owns the first catalog-driven occupancy cells,
generated physical connection cells, reservations, glued exits, and
dangling-exit diagnostics. Assembly defaults to four-way grid connectivity;
use `--connectivity eight-way` only for games where diagonal contact is meant
to count as reachable.

The initial metadata-only fixture catalog is:

```text
fixtures/shape-catalogs/2d-basic.json
fixtures/packs/2d-basic/procgen-pack.json
```

The viewer Catalog tab renders the active shape catalog from
`shapeCatalogRef`, including each build piece's footprint, reserved cells,
exits, sockets, transforms, and tags. Treat this as the visible contract for
whether build pieces are first-class inputs instead of hidden placement
side-effects.

## Pattern Catalog

The next graph grammar vocabulary is documented in:

```text
docs/v2-graph-grammar-catalog.md
fixtures/rule-catalog/v2-graph-patterns.json
```

Implemented `graph apply-rule --rule <id>` values should stay aligned with the
catalog ids and preserve the documented invariants, scoring hints, and repair
hints.

## Agent Construction Loop

The next workbench layer is tracked in:

```text
docs/agent-construction-loop.md
```

That document defines the intended external-agent loop and the planned command
surfaces for rule metadata, JSON graph summaries, candidate forking, repair
reports, data-driven batch profiles, and viewer context panes.

## Broken Fixture Check

This intentionally fails with stable diagnostics:

```bash
npm run procgen -- validate graph \
  --state fixtures/candidates/invalid-missing-key.candidate.json \
  --out artifacts/manual/invalid.validation.json
```

Expected fatal diagnostic code:

```text
required_item_unavailable
```

To turn diagnostics into an advisory repair artifact:

```bash
npm run procgen -- repair suggest \
  --state fixtures/candidates/invalid-missing-key.candidate.json \
  --out artifacts/manual/invalid.repair.json
```

Repair reports preserve validator diagnostics and add `suggestedActions`.
Suggestions are planning aids only; validate repaired candidates before scoring
or accepting them.

Some diagnostics can now be handled with bounded repair actions:

```bash
npm run procgen -- repair apply \
  --state <candidate.json> \
  --action add_rejoin_edge \
  --target <terminal-node-id> \
  --seed <u64> \
  --out <candidate.json> \
  --receipt <receipt.json>
```

## LAN Viewer

Use `den-serve` so the viewer is reachable from another machine on the LAN:

```bash
den-serve up asha-procgen -repo /home/dev/asha-procgen
```

The LAN URL printed by `den-serve` is the URL to give the human.

Useful commands:

```bash
den-serve status asha-procgen -repo /home/dev/asha-procgen
den-serve logs asha-procgen -repo /home/dev/asha-procgen
den-serve stop asha-procgen -repo /home/dev/asha-procgen
```

Serving semantics come from Den document `den-services/den-serve-agent-usage`.
Do not replace this with localhost-only instructions.

Viewer API routes:

- `/api/artifacts/first-run`
- `/api/batches/v2`
- `/api/artifacts/by-path?path=<artifact-ref-from-selection-report>`
- `/api/evidence/native-voxel-extrusion`

The batch viewer shows candidate scores, profile sequence, artifact refs,
validation status, provenance steps, and any diagnostics/repair hints for the
selected artifact. Its Build tab renders catalog piece placements when
`piecePlacementRef` is present: occupied cells, connection cells, reserved
cells, glued exits, piece labels, and socket/content markers.
Its Catalog tab renders the active build-piece shape catalog when
`shapeCatalogRef` is present.
Its Voxel tab uses the same deterministic extrusion compiler as
`npm run voxel:asha-smoke` to render an isometric floor/wall/ceiling cutaway.
The compiler consumes the placement's versioned clearance/wall/doorway policy,
keeps piece ownership through validation, and turns only glued-exit-owned
connection routes into openings.
When the selected placement matches the committed native evidence, the tab also
shows the Rust authority voxel-state hash and engine pin. Other candidates are
clearly labelled as unverified voxel proposals.
Its separate Voxel 3D tab sends a ceiling-free floor/wall projection through
the public engine-owned inspection renderer and procedural grid. Drag with the
primary mouse button or use Arrow keys to orbit; focus the canvas and use
W/A/S/D to move, and use +/− or the wheel to zoom. Candidate changes atomically
replace the retained frame and grid. The downstream projection compacts
same-material cells into deterministic maximal cuboids, and its smoke expands
those cuboids cell-by-cell to prove exact, non-overlapping material coverage for
every accepted sample before checking the public renderer frame-op limit. This
tab is projection-only inspection and does not claim runtime, collision,
navigation, native-render, or performance authority.

Build, Voxel, and Voxel 3D show the placement-policy experiment panel. The
editable values are `minimumClearanceCells` and `wallThicknessCells`; schema v1
keeps `contactPolicy=glued_exits_only`, `doorwayWidthCells=1`, and
`preservePieceBoundaries=true` locked. Apply posts only the selected candidate
ID and typed policy to the local viewer server. The server derives piece-plan,
shape-match, and catalog refs from the committed batch report, reruns Rust
assembly plus placement validation without a shell, returns an explicitly
temporary/non-native result, and removes its temporary files. Reset or candidate
switching restores the committed placement. The endpoint accepts no browser
filesystem paths and does not mutate fixtures, samples, RuntimeSession, or
native voxel evidence.

## Verification

```bash
npm run verify
```

The default gate checks ASHA dependency boundaries, TypeScript, Rust compile,
Rust tests, publishing, and a focused two-room voxel-boundary smoke. Browser
smoke is not part of the default gate yet.

For optional preview-site evidence:

```bash
npm run viewer:smoke
```

The focused placement-policy endpoint contract can be checked separately:

```bash
npm run policy:smoke
```

The standalone HTML preview smoke alias is:

```bash
npm run preview:smoke
```

This builds the viewer, starts the local preview server on `127.0.0.1`, checks
the sample batch and intermediate artifact API, verifies the dark theme CSS, and
checks the top generated standalone HTML preview for dark styling, SVG room and
corridor elements, and required content labels. It also checks the viewer Build
tab for the catalog piece placement grid, rendered cells, socket markers, and
glued-exit links, checks the Voxel tab for exposed isometric faces plus a
matching native authority receipt, and exercises the engine-owned Voxel 3D
mount, ceiling omission, public picking, and candidate frame replacement. It
uses Chromium to write layout/intermediate/build/voxel/voxel-3d/standalone-preview
screenshots plus a report under:

```text
/tmp/asha-procgen-viewer-smoke/
```

## Current Non-Goals

- No in-repo LLM harness.
- No custom agent service.
- No runtime-backed 3D authority; Voxel 3D is an engine-rendered, projection-only
  inspection of deterministic downstream build data.
- No large accepted-layout corpus yet.
- No doorway widths above one in placement-policy schema v1. Wider values fail
  closed until Rust routing validates the complete oriented opening footprint.
