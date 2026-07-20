import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';

import { buildVoxelInspectionProjection } from '../dist/ts/src/voxel-inspection-projection.js';
import { compilePlacementExtrusion } from '../dist/ts/src/voxel-extrusion.js';

const repoRoot = resolve(import.meta.dirname, '..');
const selection = await readJson('artifacts/samples/batch-v2/selection-report.json');
const entries = selection.accepted.filter((entry) => typeof entry.piecePlacementRef === 'string');
if (entries.length < 2) {
  throw new Error('voxel inspection smoke requires two accepted piece placements');
}

const projections = [];
for (const entry of entries.slice(0, 2)) {
  const placement = await readJson(entry.piecePlacementRef);
  const plan = compilePlacementExtrusion(placement);
  const projection = buildVoxelInspectionProjection(plan);
  const repeated = buildVoxelInspectionProjection(plan);
  if (JSON.stringify(projection.frame) !== JSON.stringify(repeated.frame)) {
    throw new Error(`inspection frame for ${plan.placementId} is not deterministic`);
  }
  if (projection.omittedCeilingVoxelCount <= 0) {
    throw new Error(`inspection frame for ${plan.placementId} omitted no ceiling voxels`);
  }
  if (projection.projectedVoxelCount + projection.omittedCeilingVoxelCount !== plan.solidVoxelCount) {
    throw new Error(`inspection voxel accounting does not match ${plan.placementId}`);
  }
  const creates = projection.frame.ops.filter((op) => op.op === 'create');
  if (creates.length !== projection.projectedNodeCount) {
    throw new Error(`inspection frame for ${plan.placementId} has unexpected create count`);
  }
  const representedVoxelCount = creates.reduce((count, op) =>
    count + op.node.transform.scale[0] * op.node.transform.scale[1] * op.node.transform.scale[2], 0);
  if (representedVoxelCount !== projection.projectedVoxelCount) {
    throw new Error(`inspection frame for ${plan.placementId} does not represent every projected voxel`);
  }
  for (const op of creates) {
    const maxY = op.node.transform.translation[1] + op.node.transform.scale[1] / 2;
    if (maxY > projection.ceilingY) {
      throw new Error(`inspection frame for ${plan.placementId} contains a ceiling voxel`);
    }
  }
  if (projection.frame.ops.filter((op) => op.op === 'createLight').length !== 2) {
    throw new Error(`inspection frame for ${plan.placementId} is missing engine light descriptors`);
  }
  projections.push(projection);
}

if (
  projections[0].placementId === projections[1].placementId
  || JSON.stringify(projections[0].frame) === JSON.stringify(projections[1].frame)
) {
  throw new Error('candidate switching did not produce a distinct deterministic inspection frame');
}

const viewerSource = await readFile(resolve(repoRoot, 'viewer/app.ts'), 'utf8');
const projectionSource = await readFile(resolve(repoRoot, 'src/voxel-inspection-projection.ts'), 'utf8');
if (!/from '@asha\/renderer-host'/.test(viewerSource)) {
  throw new Error('viewer does not import the engine renderer host from its package root');
}
const forbiddenRendererImport = /(?:from\s+|import\()['"](?:three(?:\/|['"])|@asha\/renderer-three)/;
if (forbiddenRendererImport.test(`${viewerSource}\n${projectionSource}`)) {
  throw new Error('voxel inspection source contains a direct renderer implementation import');
}

console.log(
  `voxel inspection frame smoke passed; ${projections[0].projectedVoxelCount} voxels in ${projections[0].projectedNodeCount} nodes, ${projections[0].omittedCeilingVoxelCount} ceiling voxels omitted`,
);

async function readJson(relativePath) {
  return JSON.parse(await readFile(resolve(repoRoot, relativePath), 'utf8'));
}
