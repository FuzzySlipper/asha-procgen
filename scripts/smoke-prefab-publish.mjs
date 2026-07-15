#!/usr/bin/env node
import assert from 'node:assert/strict';
import { mkdir, readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { serializeAshaPrefabRegistrySource } from '@asha/game-workspace';
import {
  ProcgenPublishError,
  compileProcgenPrefabPublication,
} from '../dist/ts/src/prefab-publishing.js';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const readJson = async (relativePath) => JSON.parse(await readFile(path.join(repoRoot, relativePath), 'utf8'));
const catalog = await readJson('fixtures/shape-catalogs/2d-basic.json');
const shapeMatch = await readJson('artifacts/samples/batch-v2/candidate-000/piece-shape-match.json');
const placement = await readJson('artifacts/samples/batch-v2/candidate-000/piece-placement.json');
const configuration = await readJson('fixtures/prefab-mappings/first-slice.json');
const compile = (overrides = {}) => compileProcgenPrefabPublication({
  catalog: overrides.catalog ?? catalog,
  shapeMatch: overrides.shapeMatch ?? shapeMatch,
  placement: overrides.placement ?? placement,
  configuration: overrides.configuration ?? configuration,
});

const publication = compile();
assert.equal(publication.prefabRegistry.definitions.length, 2);
assert.equal(publication.prefabInstancesArtifact.prefabInstances.length, 2);
assert.deepEqual(
  publication.prefabInstancesArtifact.prefabInstances.map((instance) => [instance.instance, instance.prefab]),
  [[2001, 1001], [2002, 1002]],
);
assert.deepEqual(publication.prefabInstancesArtifact.prefabInstances[1].transform.rotation, [0, 1, 0, 0]);
assert.equal(publication.sceneArtifact.nodes.length, 2);
assert.equal(publication.sceneArtifact.nodes[0].kind.kind, 'voxelVolume');
assert.equal(publication.manifest.artifacts.find((artifact) => artifact.role === 'prefabRegistry')?.path, 'prefabs/registry.json');
assert.equal(publication.manifest.artifacts.filter((artifact) => artifact.role === 'procgenPrefabSource').length, 2);
assert.equal(publication.provenance.instances[0].shapeId, 'shape.room.standard.1_exit');
assert.equal(publication.provenance.instances[0].matchScore, 1049);
assert.match(serializeAshaPrefabRegistrySource(publication.prefabRegistry), /"schemaVersion": 1/);

const repeatedShapeConfiguration = {
  ...configuration,
  selectedInstanceIds: [
    'instance.piece_room_room_region_hub_central_1',
    'instance.piece_room_room_region_junction_merge_1',
  ],
  mappings: [{
    ...configuration.mappings[0],
    shapeId: 'shape.room.hub.4_exit',
    prefabId: 1003,
    source: { kind: 'voxelObject', asset: 'voxel-object/procgen-standard-room' },
  }],
  instanceIdentities: [
    { procgenInstanceId: 'instance.piece_room_room_region_hub_central_1', prefabInstanceId: 2010 },
    { procgenInstanceId: 'instance.piece_room_room_region_junction_merge_1', prefabInstanceId: 2009 },
  ],
};
const repeatedShapePublication = compile({ configuration: repeatedShapeConfiguration });
assert.equal(repeatedShapePublication.prefabRegistry.definitions.length, 1);
assert.equal(repeatedShapePublication.sceneArtifact.dependencies.length, 1);
assert.deepEqual(
  repeatedShapePublication.sceneArtifact.nodes.map((node) => [node.id, node.transform.translation]),
  [[2010, [13, 0, 9]], [2009, [12, 0, 16]]],
);

expectFailure('missingPrefabMapping', () => compile({
  configuration: { ...configuration, mappings: configuration.mappings.slice(1) },
}));
expectFailure('missingStableRole', () => compile({
  configuration: {
    ...configuration,
    mappings: configuration.mappings.map((mapping, index) =>
      index === 0 ? { ...mapping, stableRole: '' } : mapping),
  },
}));
expectFailure('incompatibleSourceAsset', () => compile({
  configuration: {
    ...configuration,
    mappings: configuration.mappings.map((mapping, index) =>
      index === 0
        ? { ...mapping, source: { kind: 'voxelObject', asset: 'scene/procgen-standard-room' } }
        : mapping),
  },
}));
expectFailure('duplicateInstanceIdentity', () => compile({
  configuration: {
    ...configuration,
    instanceIdentities: configuration.instanceIdentities.map((identity) => ({
      ...identity,
      prefabInstanceId: 2001,
    })),
  },
}));
expectFailure('invalidTransform', () => compile({
  placement: {
    ...placement,
    instances: placement.instances.map((instance, index) =>
      index === 0 ? { ...instance, origin: { ...instance.origin, x: Number.NaN } } : instance),
  },
}));

const evidence = {
  ...publication,
  proof: {
    adapter: 'compileProcgenPrefabPublication',
    publicPackages: ['@asha/contracts', '@asha/game-workspace'],
    failClosedCases: [
      'missingPrefabMapping',
      'missingStableRole',
      'incompatibleSourceAsset',
      'duplicateInstanceIdentity',
      'invalidTransform',
    ],
  },
};
const evidencePath = path.join(repoRoot, 'artifacts/evidence/prefab-project-bundle-publication.json');
await mkdir(path.dirname(evidencePath), { recursive: true });
await writeFile(evidencePath, `${JSON.stringify(evidence, null, 2)}\n`);
console.log(JSON.stringify({
  evidencePath: path.relative(repoRoot, evidencePath),
  prefabDefinitions: publication.prefabRegistry.definitions.length,
  prefabInstances: publication.prefabInstancesArtifact.prefabInstances.length,
  projectBundleArtifacts: publication.manifest.artifacts.length,
  failClosedCases: evidence.proof.failClosedCases,
}, null, 2));

function expectFailure(code, callback) {
  assert.throws(callback, (error) => error instanceof ProcgenPublishError && error.code === code);
}
