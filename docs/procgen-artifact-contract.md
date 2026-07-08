# Procgen Artifact Contract

Status: graph grammar, batch selection, and intermediate layout intent contract.

The CLI workbench is file-oriented. Every command reads explicit inputs, writes
explicit outputs, and produces structured JSON a human or agent can inspect.

## Command Pattern

```bash
npm run procgen -- <command> --state <candidate.json> --out <output.json> --receipt <receipt.json> --seed <u64>
```

Use `--transcript <path>` on mutating commands when building an auditable run.

Exit code `0` means the command completed successfully. Validation failures are
written as JSON diagnostics; malformed input, IO failure, and rejected mutating
operations return non-zero.

## Candidate

Kind: `asha_procgen.candidate.v1`

The candidate is dimension-agnostic at the graph layer. The first implementation
uses `dimensionModel: "topology_graph"` and later commands may add 2D or 3D
layout artifacts without changing the graph contract.

Important fields:

- `candidateId`: stable generated id.
- `seed`: source seed.
- `sourceIntent`: seed-intent id.
- `provenance`: ordered command history.
- `graph.nodes`: intent nodes.
- `graph.edges`: directed intent edges.

Node kinds:

- `start`
- `goal`
- `gate`
- `key`
- `treasure`
- `shortcut`
- `secret`
- `hazard`
- `resource`
- `junction`

Edge kinds:

- `critical_path`
- `key_branch`
- `optional_branch`
- `shortcut`
- `secret_bypass`

Traversal kinds:

- `open`
- `locked`
- `one_way_return`
- `hidden`

Locked edges use `requiredItem`. Key nodes use `grantsItem`.

## Rule Catalog

Kind: `asha_procgen.rule_catalog.v1`

The v2 graph grammar catalog lives at:

```text
fixtures/rule-catalog/v2-graph-patterns.json
```

The companion design document is:

```text
docs/v2-graph-grammar-catalog.md
```

Pattern ids in this catalog should match future `graph apply-rule --rule <id>`
values. The catalog records required node/edge kinds, tags, validator
invariants, scoring hints, repair hints, and 2D/3D embedding notes.

Implemented v2 rule ids:

- `hub_spoke_cluster`
- `nested_lock_key_chain`
- `hazard_resource_tradeoff`
- `boss_preparation_loop`
- `gated_treasure_branch`
- `branch_merge_shortcut`

## Receipt

Kind: `asha_procgen.receipt.v1`

Receipts record command status, seed, input/output hashes, output file refs, and
diagnostics. Receipts are the primary tool-call evidence for agent transcripts.

## Validation Report

Kind: `asha_procgen.validation.graph.v1`

Validation reports contain:

- `ok`
- `fatalCount`
- `stateHash`
- `diagnostics`

Diagnostics may include `repairHint`. Agents should treat it as a suggested
next edit, not as proof that the edit is the only valid repair.

Stable diagnostic codes currently emitted by graph validation/rule rejection:

- `start_count_invalid`
- `goal_count_invalid`
- `edge_from_missing`
- `edge_to_missing`
- `required_item_unavailable`
- `goal_unreachable`
- `locked_edge_never_traversed`
- `non_goal_dead_end`
- `orphan_node`
- `hub_incident_edges_low`
- `hub_missing_wayfinding_anchor`
- `hub_missing_return_or_rejoin`
- `boss_missing_preparation`
- `boss_preparation_missing_return`
- `hazard_missing_rejoin`
- `merge_upstream_routes_low`
- `rule_already_applied`
- `missing_required_pattern`

Fatal diagnostics block acceptance. Warnings are advisory.

## Graph Analysis Report

Kind: `asha_procgen.graph_analysis.v1`

Graph analysis reports contain:

- `criticalPath`
- `dominators`
- `optionalBranches`
- `lockKeyOrder`
- `loopSignals`
- `shortcutBypassRisks`

They are intended as agent planning context, not as validation authority.

## Rule Compatibility Report

Kind: `asha_procgen.rule_compatibility.v1`

Compatibility reports list every known graph rule with one of:

- `applicable`
- `blocked`
- `duplicate`
- `risky`

Each entry may include reasons and recommended actions.

## Spatial Intent Report

Kind: `asha_procgen.spatial_intent.v1`

Spatial intent reports annotate graph nodes and edges with pre-geometry hints
such as `landmark_hub`, `visible_before_reachable`, `pressure_path`,
`shortcut_connector`, `one_way_drop`, and `hidden_route`.

## Intermediate Breakdown

Kind: `asha_procgen.intermediate_breakdown.v1`

Intermediate breakdowns contain:

- `regions`: graph-derived region roles and optional anchor nodes
- `connectors`: graph-edge-derived connector intents
- `constraints`: named constraints that later geometry passes should preserve

Validation uses kind `asha_procgen.validation.intermediate.v1`. This schema is
intentionally not a 2D room layout, 3D prefab graph, mesh, voxel grid, or tile
map. See `docs/intermediate-layout-contract.md`.

## Score Report

Kind: `asha_procgen.score.graph.v1`

First-slice metrics:

- `nodeCount`
- `edgeCount`
- `criticalPathLength`
- `loopCount`
- `optionalBranchCount`
- `lockedEdgeCount`
- `shortcutCount`
- `deadEndCount`
- `hubCount`
- `wayfindingAnchorCount`
- `preparationCount`
- `hazardCount`
- `bossCount`
- `mergeCount`
- `pressureEdgeCount`
- `rejoinEdgeCount`

`overall` is a deterministic heuristic score, not a final design verdict.

## Selection Report

Kind: `asha_procgen.selection_report.v1`

Batch generation writes:

```text
artifacts/samples/batch-v2/selection-report.json
```

The report contains:

- `batchId`
- `seed`
- `requestedCount`
- `generatedCount`
- `accepted`: sorted accepted candidates with artifact, validation, score, and
  layout refs
- `rejected`: rejected candidate refs plus diagnostics

Accepted entries include:

- `topologyFingerprint`
- `duplicateOf`
- `budgetChecks`
- `budgetPenalty`
- `selectionScore`
- `analysisRef`
- `compatibleRulesRef`
- `spatialIntentRef`
- `intermediateBreakdownRef`
- `intermediateValidationRef`

Accepted entries are sorted by descending `selectionScore`, then candidate id
for stable tie-breaking.

## Layout Artifact

Kind: `asha_procgen.layout_2d.v1`

The first layout artifact is an inspectable 2D embedding. It preserves graph
node and edge IDs so diagnostics and viewer labels map back to intent. It is
not a renderer or final tile map.

## Accepted Artifact

Kind: `asha_procgen.accepted_artifact.v1`

Accepted artifacts bundle the candidate, layout, score summary, hashes, and
validation/score refs. They are suitable for later catalog and shuffle-bag work.

## Transcript

Transcript files are JSONL. Each line is a `tool_event` with command, output
state, receipt, seed, and args.

Example:

```json
{"kind":"tool_event","command":"graph apply-rule","state":"artifacts/samples/first-run/candidate-001-lock_key_loop.json","receipt":"artifacts/samples/first-run/receipt-001-lock_key_loop.json","seed":4104,"args":{"rule":"lock_key_loop"}}
```
