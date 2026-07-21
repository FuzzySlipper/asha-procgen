#!/usr/bin/env node
import assert from 'node:assert/strict';

import { compilePlacementExtrusion } from '../dist/ts/src/voxel-extrusion.js';

const policy = {
  schemaVersion: 1,
  minimumClearanceCells: 3,
  contactPolicy: 'glued_exits_only',
  wallThicknessCells: 1,
  doorwayWidthCells: 1,
  preservePieceBoundaries: true,
};
const occupiedCells = [
  ...ownedSquare('instance.room_a', 0, 0),
  ...ownedSquare('instance.room_b', 5, 0),
];
const connectionOwner = 'connection.glue_room_a_room_b';
const placement = {
  kind: 'asha_procgen.piece_placement.v1',
  placementId: 'piece_placement.boundary_smoke',
  gridConnectivity: 'four_way',
  placementPolicy: policy,
  occupiedCells,
  connectionCells: [2, 3, 4].map((x) => ({ instanceId: connectionOwner, x, y: 0 })),
  gluedExits: [{
    id: 'glue.room_a.room_b',
    fromInstance: 'instance.room_a',
    toInstance: 'instance.room_b',
  }],
};

const plan = compilePlacementExtrusion(placement);
assert.equal(plan.openingCellCount, 3);
assert.equal(plan.walkableCellCount, 11);
for (const x of [2, 3, 4]) {
  assert.ok(hasSolid(plan, x, 0, 0, 2), `doorway route ${x},0 is missing its floor`);
  assert.ok(hasSolid(plan, x, 1, 1, 1), `wall line beside doorway route ${x},0 is missing`);
  assert.equal(hasSolid(plan, x, 1, 0, 1), false, `doorway route ${x},0 was closed by a wall`);
}
assert.ok(hasSolid(plan, 2, 1, 1, 1), 'room-facing wall is missing outside the declared opening');
assert.ok(hasSolid(plan, 4, 1, 1, 1), 'second room-facing wall is missing outside the declared opening');

assert.throws(
  () => compilePlacementExtrusion({
    ...placement,
    occupiedCells: [
      ...ownedSquare('instance.room_a', 0, 0),
      ...ownedSquare('instance.room_b', 3, 0),
    ],
  }),
  /piece boundary clearance 3 violated/,
);
assert.throws(
  () => compilePlacementExtrusion({
    ...placement,
    placementPolicy: { ...policy, preservePieceBoundaries: false },
  }),
  /requires preservePieceBoundaries=true/,
);
assert.throws(
  () => compilePlacementExtrusion({
    ...placement,
    placementPolicy: { ...policy, wallThicknessCells: 2, minimumClearanceCells: 4 },
  }),
  /at least twice wallThicknessCells plus one/,
);
assert.throws(
  () => compilePlacementExtrusion({
    ...placement,
    connectionCells: [{ instanceId: 'connection.undeclared', x: 2, y: 0 }],
  }),
  /not owned by a declared glued exit/,
);

console.log(
  `voxel boundary smoke passed; ${plan.walkableCellCount} walkable cells, ${plan.openingCellCount} declared opening cells, ${plan.boundaryCellCount} boundary cells`,
);

function ownedSquare(instanceId, minX, minY) {
  return [
    { instanceId, x: minX, y: minY },
    { instanceId, x: minX + 1, y: minY },
    { instanceId, x: minX, y: minY + 1 },
    { instanceId, x: minX + 1, y: minY + 1 },
  ];
}

function hasSolid(plan, x, y, z, material) {
  return plan.solidVoxels.some((voxel) => (
    voxel.coord.x === x
    && voxel.coord.y === y
    && voxel.coord.z === z
    && voxel.material === material
  ));
}
