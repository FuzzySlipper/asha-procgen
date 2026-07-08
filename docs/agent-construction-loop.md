# Agent Construction Loop

Status: north-star contract for task series #4886.

This repo is intentionally a file-oriented procgen workbench, not an agent
service. External harnesses should be able to steer level construction by
calling deterministic CLI tools, reading JSON artifacts, and discarding bad
candidates without trusting an LLM as validator or authority.

## Loop Shape

The intended agent loop is:

```text
inspect rules -> inspect candidate -> fork -> apply rule -> validate -> repair or score -> embed -> accept or discard
```

The important boundary is that the agent chooses actions, but the CLI owns the
state mutation and the validators own pass/fail evidence.

## Current Commands

Already available:

```bash
npm run procgen -- init --intent <intent.json> --seed <u64> --out <candidate.json> --receipt <receipt.json>
npm run procgen -- graph apply-rule --state <candidate.json> --rule <rule_id> --seed <u64> --out <candidate.json> --receipt <receipt.json>
npm run procgen -- graph summarize --state <candidate.json>
npm run procgen -- validate graph --state <candidate.json> --out <validation.json>
npm run procgen -- score graph --state <candidate.json> --out <score.json>
npm run procgen -- embed 2d --state <candidate.json> --seed <u64> --out <layout.json> --receipt <receipt.json>
npm run procgen -- accept --candidate <candidate.json> --layout <layout.json> --validation <validation.json> --score <score.json> --out <accepted.json> --receipt <receipt.json>
npm run procgen -- batch generate --out-dir <dir> --seed <u64> --count <n>
```

Implemented graph rules:

```text
lock_key_loop
optional_treasure_detour
one_way_shortcut
secret_bypass
hub_spoke_cluster
nested_lock_key_chain
hazard_resource_tradeoff
boss_preparation_loop
gated_treasure_branch
branch_merge_shortcut
```

## Planned Agent Surfaces

These are the next command/artifact surfaces for #4886.

### Rule Metadata

Implemented command:

```bash
npm run procgen -- graph rules --out artifacts/manual/rules.json
```

Artifact kind:

```text
asha_procgen.rule_metadata.v1
```

The artifact includes rule id, intent, emitted node/edge tags, required existing
patterns, duplicate marker ids, compatibility hints, and repair hints. This lets
an agent choose a plausible next rule before mutating state.

### Candidate Summary

Implemented command:

```bash
npm run procgen -- graph summarize --state <candidate.json> --json --out <summary.json>
```

Artifact kind:

```text
asha_procgen.graph_summary.v1
```

The summary includes node/edge counts, tags, locked items, dead ends, score
metrics, validation status, node/edge summaries, and a short provenance tail. It
is small enough to paste into an agent context window for most current
candidates.

### Candidate Fork

Goal command:

```bash
npm run procgen -- graph fork --state <candidate.json> --label <name> --seed <u64> --out <candidate.json> --receipt <receipt.json>
```

Forking should preserve the graph and provenance, add a fork provenance step,
and produce a receipt/transcript event. Agents should use this instead of shell
copying candidates when trying alternate plans.

### Repair Report

Goal command:

```bash
npm run procgen -- repair suggest --state <candidate.json> --out <repair.json>
```

Artifact kind:

```text
asha_procgen.repair_report.v1
```

The report should sort validation diagnostics by severity, preserve
`repairHint`, and optionally include known next tool actions such as
`apply lock_key_loop before nested_lock_key_chain`. Suggestions are advisory:
they do not replace validation.

### Batch Profile Fixture

Goal command:

```bash
npm run procgen -- batch generate --profile fixtures/batch-profiles/v2-sample.json --out-dir <dir> --seed <u64> --count <n>
```

Artifact kind:

```text
asha_procgen.batch_profile.v1
```

Profiles should move rule sequences, weights, and selection labels into JSON so
external agents can propose generation strategies without editing Rust.

### Viewer Context Panes

The static viewer should remain LAN-first through `den-serve`. The next viewer
layer should show, for the selected batch candidate:

- provenance steps;
- validation diagnostics and repair hints;
- score metrics;
- artifact refs;
- rule/tag summary.

## Agent Guidance

Prefer short, reversible steps:

- inspect rule metadata before choosing a rule;
- fork before applying a risky or incompatible rule;
- validate immediately after structural changes;
- use repair reports to pick the next bounded action;
- score only valid candidates;
- accept only candidates with validation and score refs.

Batch generation should be used as a cheap search tool, not as proof that all
accepted candidates are good game levels. Selection scores are deterministic
heuristics for triage.

## Boundaries

This series does not add:

- an in-repo LLM harness;
- a socket/server orchestration API;
- hidden mutable workspace state;
- 3D embedding;
- voxel output;
- ASHA runtime/renderer integration;
- imports from ASHA internals.

All durable state should remain in explicit files under the caller-chosen output
directory.
