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
- `candidate-000/score.graph.json`
- `candidate-000/transcript.jsonl`

The sample command generates 10 candidates from:

```text
fixtures/batch-profiles/v2-sample.json
```

The selection report records the profile id/ref, the profile sequence used for
each candidate, and sorts accepted entries by deterministic score.

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

The batch viewer shows candidate scores, profile sequence, artifact refs,
validation status, provenance steps, and any diagnostics/repair hints for the
selected artifact.

## Verification

```bash
npm run verify
```

The default gate checks ASHA dependency boundaries, TypeScript, Rust compile, and
Rust tests. Browser smoke is not part of the default gate yet.

## Current Non-Goals

- No in-repo LLM harness.
- No custom agent service.
- No ASHA runtime or renderer integration.
- No Daggerfall-style 3D embedding yet.
- No large accepted-layout corpus yet.
