# Agent Runbook

Status: first-slice runbook for the CLI workbench and viewer.

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
