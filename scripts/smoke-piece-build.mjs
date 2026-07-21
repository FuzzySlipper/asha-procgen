import { mkdtempSync, readFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';

const tmp = mkdtempSync(join(tmpdir(), 'asha-procgen-piece-build-'));

const paths = {
  catalogReport: join(tmp, 'shape-catalog.report.json'),
  piecePlan: join(tmp, 'piece-plan.json'),
  shapeMatch: join(tmp, 'piece-shape-match.json'),
  placement: join(tmp, 'piece-placement.json'),
  validation: join(tmp, 'piece-placement.validation.json'),
};

function runProcgen(args) {
  const result = spawnSync('npm', ['run', 'procgen', '--', ...args], {
    encoding: 'utf8',
    stdio: 'pipe',
  });
  if (result.status !== 0) {
    process.stderr.write(result.stdout);
    process.stderr.write(result.stderr);
    process.exit(result.status ?? 1);
  }
}

runProcgen([
  'build',
  'catalog',
  'inspect',
  '--catalog',
  'fixtures/shape-catalogs/2d-basic.json',
  '--out',
  paths.catalogReport,
]);
runProcgen([
  'build',
  'emit-piece-plan',
  '--candidate',
  'artifacts/samples/batch-v2/candidate-000/candidate-003-branch_merge_shortcut.json',
  '--intermediate',
  'artifacts/samples/batch-v2/candidate-000/intermediate-breakdown.json',
  '--geometry',
  'artifacts/samples/batch-v2/candidate-000/geometry-2d.json',
  '--out',
  paths.piecePlan,
]);
runProcgen([
  'build',
  'match-shapes',
  '--catalog',
  'fixtures/shape-catalogs/2d-basic.json',
  '--piece-plan',
  paths.piecePlan,
  '--seed',
  '7101',
  '--out',
  paths.shapeMatch,
]);
runProcgen([
  'build',
  'assemble',
  '--catalog',
  'fixtures/shape-catalogs/2d-basic.json',
  '--piece-plan',
  paths.piecePlan,
  '--shape-match',
  paths.shapeMatch,
  '--out',
  paths.placement,
]);
runProcgen([
  'build',
  'validate-placement',
  '--state',
  paths.placement,
  '--out',
  paths.validation,
]);

const catalog = JSON.parse(readFileSync(paths.catalogReport, 'utf8'));
const match = JSON.parse(readFileSync(paths.shapeMatch, 'utf8'));
const placement = JSON.parse(readFileSync(paths.placement, 'utf8'));
const validation = JSON.parse(readFileSync(paths.validation, 'utf8'));

if (catalog.diagnostics.length !== 0) {
  throw new Error(`catalog inspect emitted ${catalog.diagnostics.length} diagnostic(s)`);
}
if (!match.ok || match.unmatchedCount !== 0) {
  throw new Error(`shape match failed with ${match.unmatchedCount} unmatched requirement(s)`);
}
if (!validation.ok) {
  throw new Error(`placement validation failed with ${validation.fatalCount} fatal diagnostic(s)`);
}
if (placement.gridConnectivity !== 'four_way') {
  throw new Error(`unexpected placement connectivity: ${placement.gridConnectivity}`);
}
if (
  placement.placementPolicy?.contactPolicy !== 'glued_exits_only'
  || placement.placementPolicy?.preservePieceBoundaries !== true
  || placement.placementPolicy?.minimumClearanceCells
    < placement.placementPolicy?.wallThicknessCells * 2 + 1
) {
  throw new Error(`placement emitted an unsafe boundary policy: ${JSON.stringify(placement.placementPolicy)}`);
}
if (JSON.stringify(placement.placementPolicy) !== JSON.stringify(catalog.placementPolicy)) {
  throw new Error('placement policy does not match the inspected shape catalog policy');
}
if (!Array.isArray(placement.connectionCells) || placement.connectionCells.length === 0) {
  throw new Error('placement emitted no connection cells');
}
for (const glued of placement.gluedExits) {
  const owner = `connection.${slugifyLabel(glued.id)}`;
  const routed = new Set(placement.connectionCells
    .filter((cell) => cell.instanceId === owner)
    .map((cell) => `${cell.x},${cell.y}`));
  if (
    !routed.has(`${glued.fromCell.x},${glued.fromCell.y}`)
    || !routed.has(`${glued.toCell.x},${glued.toCell.y}`)
  ) {
    throw new Error(`placement route ${owner} omitted a transformed catalog exit endpoint`);
  }
}

console.log(
  JSON.stringify({
    tmp,
    catalog: catalog.catalogId,
    shapes: catalog.shapeCount,
    matches: match.matches.length,
    instances: placement.instances.length,
    gluedExits: placement.gluedExits.length,
    occupiedCells: placement.occupiedCells.length,
    connectionCells: placement.connectionCells.length,
    placementPolicy: placement.placementPolicy,
    gridConnectivity: placement.gridConnectivity,
    validationOk: validation.ok,
  }),
);

function slugifyLabel(label) {
  const slug = label
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '');
  return slug.length === 0 ? 'fork' : slug;
}
