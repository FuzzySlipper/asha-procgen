import { renderHandle, type RenderDiff, type RenderFrameDiff } from '@asha/contracts';

import type { VoxelExtrusionPlan } from './voxel-extrusion.js';

export interface VoxelInspectionProjection {
  readonly frame: RenderFrameDiff;
  readonly placementId: string;
  readonly ceilingY: number;
  readonly projectedVoxelCount: number;
  readonly projectedNodeCount: number;
  readonly omittedCeilingVoxelCount: number;
  readonly camera: {
    readonly position: readonly [number, number, number];
    readonly target: readonly [number, number, number];
    readonly moveSpeed: number;
  };
}

const MATERIAL_COLORS: Readonly<Record<number, readonly [number, number, number, number]>> = {
  1: [0.2, 0.24, 0.28, 1],
  2: [0.23, 0.58, 0.39, 1],
};
const UNKNOWN_MATERIAL_COLOR = [0.72, 0.31, 0.68, 1] as const;

interface VoxelInspectionBox {
  readonly min: readonly [number, number, number];
  readonly maxExclusive: readonly [number, number, number];
  readonly material: number;
  readonly voxelCount: number;
}

/**
 * Creates a projection-only inspection frame from the canonical extrusion plan.
 * The top enclosure layer is intentionally excluded from this presentation frame;
 * the source plan and its native evidence remain fully enclosed.
 */
export function buildVoxelInspectionProjection(plan: VoxelExtrusionPlan): VoxelInspectionProjection {
  const ceilingY = plan.buildBounds.maxExclusive.y - 1;
  const projectedVoxels = plan.solidVoxels.filter((voxel) => voxel.coord.y !== ceilingY);
  const omittedCeilingVoxelCount = plan.solidVoxels.length - projectedVoxels.length;
  if (omittedCeilingVoxelCount === 0) {
    throw new Error(`extrusion ${plan.placementId} has no ceiling layer at y=${ceilingY}`);
  }

  const boxes = compactInspectionVoxels(projectedVoxels, plan.buildBounds.min.y);
  const voxelOps: RenderDiff[] = boxes.map((box, index) => ({
    op: 'create',
    handle: renderHandle(index + 1),
    parent: null,
    node: {
      geometry: { shape: 'cube' },
      material: {
        color: MATERIAL_COLORS[box.material] ?? UNKNOWN_MATERIAL_COLOR,
        wireframe: false,
      },
      transform: {
        translation: [
          (box.min[0] + box.maxExclusive[0]) / 2,
          (box.min[1] + box.maxExclusive[1]) / 2,
          (box.min[2] + box.maxExclusive[2]) / 2,
        ],
        rotation: [0, 0, 0, 1],
        scale: [
          box.maxExclusive[0] - box.min[0],
          box.maxExclusive[1] - box.min[1],
          box.maxExclusive[2] - box.min[2],
        ],
      },
      visible: true,
      layer: 'scene',
      metadata: {
        source: null,
        tags: [],
        label: `procgen-voxel-box:${box.min.join(',')}..${box.maxExclusive.join(',')}:material-${box.material}:voxels-${box.voxelCount}`,
      },
    },
  }));
  const firstLightHandle = voxelOps.length + 1;
  const lightOps: readonly RenderDiff[] = [
    {
      op: 'createLight',
      handle: renderHandle(firstLightHandle),
      parent: null,
      light: {
        kind: 'ambient',
        color: [0.72, 0.78, 0.85],
        intensity: 0.8,
        enabled: true,
        shadowIntent: 'disabled',
      },
    },
    {
      op: 'createLight',
      handle: renderHandle(firstLightHandle + 1),
      parent: null,
      light: {
        kind: 'directional',
        color: [1, 0.93, 0.82],
        intensity: 1.6,
        enabled: true,
        direction: [-1, -2, -0.75],
        shadowIntent: 'disabled',
      },
    },
  ];

  const width = plan.buildBounds.maxExclusive.x - plan.buildBounds.min.x;
  const depth = plan.buildBounds.maxExclusive.z - plan.buildBounds.min.z;
  const height = plan.buildBounds.maxExclusive.y - plan.buildBounds.min.y;
  const centerX = plan.buildBounds.min.x + width / 2;
  const centerZ = plan.buildBounds.min.z + depth / 2;
  const radius = Math.max(width, depth, height, 8);
  const target = [centerX, plan.buildBounds.min.y + Math.min(height / 2, 2), centerZ] as const;

  return {
    frame: { ops: [...voxelOps, ...lightOps] },
    placementId: plan.placementId,
    ceilingY,
    projectedVoxelCount: projectedVoxels.length,
    projectedNodeCount: boxes.length,
    omittedCeilingVoxelCount,
    camera: {
      position: [
        centerX + radius * 0.5,
        plan.buildBounds.maxExclusive.y + radius * 0.8,
        centerZ + radius * 0.75,
      ],
      target,
      moveSpeed: Math.max(4, radius * 0.4),
    },
  };
}

function compactInspectionVoxels(
  voxels: VoxelExtrusionPlan['solidVoxels'],
  floorY: number,
): readonly VoxelInspectionBox[] {
  const groups = new Map<string, typeof voxels[number][]>();
  for (const voxel of voxels) {
    const groupKey = voxel.coord.y === floorY
      ? `floor:${voxel.coord.y}:${voxel.coord.z}:${voxel.material}`
      : `wall:${voxel.coord.x}:${voxel.coord.z}:${voxel.material}`;
    const group = groups.get(groupKey) ?? [];
    group.push(voxel);
    groups.set(groupKey, group);
  }

  const boxes: VoxelInspectionBox[] = [];
  for (const group of groups.values()) {
    const floorRun = group[0]?.coord.y === floorY;
    const sorted = [...group].sort((left, right) => floorRun
      ? left.coord.x - right.coord.x
      : left.coord.y - right.coord.y);
    let runStart = sorted[0];
    let previous = sorted[0];
    for (const voxel of sorted.slice(1)) {
      const contiguous = floorRun
        ? voxel.coord.x === (previous?.coord.x ?? 0) + 1
        : voxel.coord.y === (previous?.coord.y ?? 0) + 1;
      if (!contiguous && runStart !== undefined && previous !== undefined) {
        boxes.push(boxForRun(runStart, previous));
        runStart = voxel;
      }
      previous = voxel;
    }
    if (runStart !== undefined && previous !== undefined) {
      boxes.push(boxForRun(runStart, previous));
    }
  }
  return boxes.sort((left, right) =>
    left.min[0] - right.min[0]
    || left.min[1] - right.min[1]
    || left.min[2] - right.min[2]
    || left.material - right.material);
}

function boxForRun(
  first: VoxelExtrusionPlan['solidVoxels'][number],
  last: VoxelExtrusionPlan['solidVoxels'][number],
): VoxelInspectionBox {
  const min = [first.coord.x, first.coord.y, first.coord.z] as const;
  const maxExclusive = [last.coord.x + 1, last.coord.y + 1, last.coord.z + 1] as const;
  return {
    min,
    maxExclusive,
    material: first.material,
    voxelCount:
      (maxExclusive[0] - min[0])
      * (maxExclusive[1] - min[1])
      * (maxExclusive[2] - min[2]),
  };
}
