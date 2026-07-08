# V2 Graph Grammar Catalog

Status: contract for the next richer graph grammar series.

The first slice proved a file-oriented CLI can create, mutate, validate, score,
embed, accept, and view a simple lock/key dungeon graph. The v2 grammar should
make the generated graphs feel more like deliberate level design while keeping
the core representation dimension-agnostic.

This document defines the next pattern vocabulary. It is not an implementation
claim; CLI rule implementation is tracked separately.

Machine-readable companion:

```text
fixtures/rule-catalog/v2-graph-patterns.json
```

## Design Rules

- Patterns operate on the intent graph, not on tiles or rooms.
- Pattern ids use snake_case and should match future CLI rule ids.
- Every pattern should preserve node/edge identity so validation, scoring,
  embedding, and viewer diagnostics can map back to design intent.
- Patterns may add tags and annotations needed by later 2D or 3D embedding, but
  should not require a planar grid.
- A pattern is not good just because it adds nodes. It should create a player
  choice, promise, loop, risk, or pacing beat.

## Shared Annotation Vocabulary

Node tags:

- `critical`: required progression node.
- `optional`: branch that can be skipped.
- `reward`: branch payoff or treasure.
- `preparation`: resource, safe room, clue, or advantage before pressure.
- `hazard`: risk-bearing node.
- `wayfinding_anchor`: landmark or orientation cue.
- `merge`: branch recombination point.
- `boss`: major challenge gate or climax.

Edge tags:

- `approach`: moves toward a gate, boss, or focal node.
- `branch`: leaves the critical route.
- `return`: reconnects to known space.
- `rejoin`: merges optional path back into critical or hub structure.
- `locked`: requires a granted item/resource.
- `hidden`: discoverability constraint.
- `pressure`: cost/risk/attrition path.
- `shortcut`: path compression or return route.
- `vertical_candidate`: safe future hint for 3D embedding.

Scoring dimensions:

- `loop_value`
- `branch_value`
- `pacing_value`
- `wayfinding_value`
- `risk_reward_value`
- `critical_path_depth`
- `disorientation_budget`

## Pattern: `hub_spoke_cluster`

Intent: create a readable exploration hub with several spokes and at least one
return/merge path. This gives players local choice without losing the main
objective.

Player-facing effect: the player enters a recognizable anchor room, checks
multiple branches, and returns with better information, resources, or a route
forward.

Required structure:

- one `junction` node tagged `hub` and `wayfinding_anchor`;
- at least two spoke branches;
- at least one spoke tagged `optional`;
- at least one edge tagged `return` or `rejoin`;
- critical path must continue through or beyond the hub.

Validator invariants:

- hub has at least three incident edges;
- at least one spoke can return/rejoin;
- hub remains reachable from `start`;
- `goal` remains reachable.

Repair hints:

- add a `rejoin` edge from a spoke back to the hub;
- add a wayfinding tag to the hub;
- reduce spoke count if dead ends exceed the current budget.

Scoring:

- reward `branch_value` for each useful spoke;
- reward `wayfinding_value` for a tagged anchor;
- penalize unmerged dead ends.

2D embedding notes: place hub centrally with spokes arranged radially or in
short side branches.

3D embedding notes: spokes may become vertical connectors later, but the hub
should remain an orientation anchor.

## Pattern: `nested_lock_key_chain`

Intent: create layered progression where one key or preparation branch unlocks a
deeper gate.

Player-facing effect: the player sees a blocked goal, solves a nearby branch,
then discovers a second layer of lock/goal tension.

Required structure:

- two `gate` nodes or one `gate` plus one gated reward/boss edge;
- at least two `key` nodes granting distinct items;
- key order must be reachable before each corresponding gate is traversed.

Validator invariants:

- every locked edge has a provider node;
- provider node is reachable without the item it grants;
- lock order is not circular unless explicitly marked as puzzle-cycle and
  validated separately;
- `goal` remains reachable after collecting keys.

Repair hints:

- add missing key provider;
- move key branch before its gate;
- split circular lock dependencies.

Scoring:

- reward `critical_path_depth`;
- reward nested structure only when both locks matter;
- penalize chains that add gates without alternate branches or reveals.

2D embedding notes: place earlier key branches near the first gate reveal.

3D embedding notes: nested locks can use vertical overlooks where a later gate
is visible before it is reachable.

## Pattern: `hazard_resource_tradeoff`

Intent: make a branch ask whether the player spends time/risk to gain a resource
or advantage.

Player-facing effect: the player chooses between a risky path, a safer but
longer path, or a preparation reward before pressure.

Required structure:

- at least one `hazard` node;
- at least one `resource` node or reward node tagged `preparation`;
- branch edge tagged `pressure`;
- merge/rejoin back to progression.

Validator invariants:

- hazard branch is optional or has an alternate safe route;
- resource/preparation node is reachable before it is needed;
- branch does not become a mandatory dead end.

Repair hints:

- add a merge edge after the hazard;
- add a resource payoff before the rejoin;
- mark the branch optional if it is not required progression.

Scoring:

- reward `risk_reward_value` when hazard and resource are paired;
- penalize hazards on mandatory critical path without preparation.

2D embedding notes: show risk path and safe/longer path in parallel.

3D embedding notes: hazard can become a vertical drop, traversal challenge, or
breakable-route cost later.

## Pattern: `boss_preparation_loop`

Intent: make a major challenge feel foreshadowed and prepared for rather than
arbitrary.

Player-facing effect: the player sees or anticipates a boss gate, gathers a
resource/clue/shortcut, then returns to challenge it.

Required structure:

- one `gate` or `junction` tagged `boss`;
- at least one preparation branch tagged `preparation`;
- a return/rejoin edge from preparation to boss approach;
- optional shortcut or safety route after the boss.

Validator invariants:

- preparation branch is reachable before boss edge;
- boss edge remains on critical path unless marked optional;
- preparation is not an orphan reward with no relation to boss approach.

Repair hints:

- add preparation branch before boss node;
- add rejoin edge from preparation to boss approach;
- tag boss gate and preparation node consistently.

Scoring:

- reward `pacing_value`;
- reward `loop_value` when preparation returns to a known boss reveal;
- penalize boss nodes with no setup.

2D embedding notes: place boss gate visible from earlier approach when possible.

3D embedding notes: boss reveal can be an overlook, sound cue, or vertical arena
glimpse.

## Pattern: `gated_treasure_branch`

Intent: create optional reward content that uses the same lock/key vocabulary as
critical progression without blocking the main path.

Player-facing effect: the player may spend attention or resources to open a
reward branch, but can still finish the dungeon without it.

Required structure:

- `treasure` node tagged `reward` and `optional`;
- locked edge or hidden edge into the branch;
- key/secret/resource provider that is not mandatory for main progression;
- branch may rejoin or may be a bounded optional endpoint.

Validator invariants:

- main `goal` is reachable without entering the branch;
- reward gate has a provider if locked;
- optional endpoint is allowed only when tagged `optional`.

Repair hints:

- mark reward branch optional;
- add missing provider;
- add rejoin edge if branch is intended as loop rather than endpoint.

Scoring:

- reward `branch_value` when reward is reachable but non-mandatory;
- penalize optional branch if it steals critical-key semantics.

2D embedding notes: place branch near the main path with visible but gated payoff.

3D embedding notes: reward can be visible through a grate, drop, or overlook.

## Pattern: `branch_merge_shortcut`

Intent: prevent branches from feeling like pure dead ends by giving them useful
recombination or path compression.

Player-facing effect: the player returns to known space with a changed mental
map, opened route, or shorter future traversal.

Required structure:

- two or more branches from a source or hub;
- merge node tagged `merge` or an edge tagged `shortcut`;
- at least one edge tagged `return`, `rejoin`, or `shortcut`.

Validator invariants:

- merge target is reachable from at least two distinct upstream routes;
- shortcut does not bypass required critical locks unless tagged as intentional;
- branch identities remain visible after merge.

Repair hints:

- add merge node;
- add shortcut edge from branch end to known route;
- add required-item check if shortcut bypasses a gate.

Scoring:

- reward `loop_value`;
- reward reduced backtracking if critical path is still valid;
- penalize accidental bypass of required progression.

2D embedding notes: draw merge/shortcut visibly so the viewer can show loop
function.

3D embedding notes: shortcut can be a drop, lift, ladder, one-way door, or
breakable connection.

## Pattern: `pacing_wayfinding_annotations`

Intent: add non-topological design metadata that helps validators, scorers,
viewers, and future embeddings judge a graph.

Player-facing effect: the dungeon gains landmarks, pressure valleys, safe
rooms, reveal points, and orientation aids.

Required structure:

- no required new topology;
- tags/annotations on nodes or edges such as `wayfinding_anchor`,
  `preparation`, `pressure`, `vertical_candidate`, `reveal`, or `safe_room`.

Validator invariants:

- at least one anchor should exist for larger graphs;
- pressure sequences should not exceed a configured budget without preparation;
- vertical/disorientation tags should remain metadata until 3D embedding exists.

Repair hints:

- add `wayfinding_anchor` to a hub or key reveal;
- insert preparation before long pressure chain;
- reduce disorientation tags until 3D validator exists.

Scoring:

- reward `wayfinding_value`;
- reward pacing alternation between pressure and preparation;
- penalize long unannotated graphs.

2D embedding notes: use labels/color in viewer for anchors and pressure beats.

3D embedding notes: annotations become constraints for vertical connectors,
overlooks, landmarks, and disorientation budget.

## Selection Goals

Batch selection should not simply choose the highest node count. The first
selection report should prefer candidates that are valid and balanced across:

- useful loops;
- meaningful optional branches;
- reachable lock/key order;
- bounded dead-end count;
- at least one pacing or wayfinding anchor in larger graphs;
- novelty against earlier accepted candidates;
- room for later 3D embedding without requiring it.

Reject summaries should preserve diagnostic codes so an external agent harness
can learn which repair strategy to try next.
